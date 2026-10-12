# Deterministic XML & Brazilian NF-e Parsing

**Proving baseline:** Rust + `roxmltree` + `rust_decimal`, tested through Astra Phase 5  
**Last verified:** 2026-09-12

This reference documents deterministic structured-document extraction patterns. It does **not** claim legal/fiscal validation by SEFAZ, XML-signature verification, or tax-compliance correctness.

---

## 1. Hostile XML Boundary

XML received from external systems can carry high-risk parser inputs:

1. DTD/entity expansion (including Billion Laughs style payloads);
2. external entity attempts;
3. extreme nesting depth;
4. malformed/truncated XML;
5. invalid control characters;
6. oversized payloads.

### Tested parser behavior

The proving implementation uses `roxmltree` and explicitly tests malicious DTD/entity input. It also layers application guards:

```rust
pub fn parse_safe_xml(xml: &str) -> Result<roxmltree::Document<'_>, StructuredParseError> {
    if xml.len() > MAX_XML_PARSE_BYTES {
        return Err(StructuredParseError::SecurityViolation(
            "XML exceeds configured size limit".into(),
        ));
    }

    validate_xml_characters(xml)?;

    let doc = roxmltree::Document::parse(xml)
        .map_err(|e| StructuredParseError::MalformedXml(e.to_string()))?;

    check_node_depth(doc.root_element(), 1)?;
    Ok(doc)
}
```

The reusable lesson is not “XML is safe because roxmltree is immune.” It is:

> Choose a parser whose behavior matches the threat model, test hostile inputs explicitly, and add application-level size/depth/character limits around it.

---

## 2. Structural Detection, Not Substring Detection

Do not classify a fiscal XML because the bytes happen to contain `NFe`.

A deterministic detector should inspect:

- root element name;
- namespace;
- expected child structure;
- required attributes such as `infNFe/@Id` and version where applicable.

Useful outcomes are explicit:

```text
recognized_nfe
recognized_nfe_proc
unsupported_xml
malformed_xml
```

This allows routing without pretending unsupported XML is invalid fiscal data.

---

## 3. Exact Decimal Arithmetic

### General financial rule

Do not use `f32` / `f64` as the authoritative representation for monetary values, tax bases, rates, quantities where exact decimal semantics matter, or invoice totals.

The proving implementation uses:

```rust
use rust_decimal::Decimal;
```

and tests edge cases such as:

```text
0.01
1.10
0.3333
1234.56
999999.99
```

The important property is deterministic decimal representation without binary floating-point drift.

---

## 4. NF-e Access-Key Validation

### Tested behavior

The parser validates:

- exactly 44 ASCII digits;
- Modulo-11 check digit;
- positional extraction of UF, AAMM, issuer CNPJ, model, series, number, emission type, cNF, and cDV.

Modulo-11 calculation:

```rust
let mut sum = 0u32;
let mut weight = 2u32;

for &digit in digits[..43].iter().rev() {
    sum += digit * weight;
    weight += 1;
    if weight > 9 {
        weight = 2;
    }
}

let remainder = sum % 11;
let expected_cd = if remainder < 2 { 0 } else { 11 - remainder };
```

A checksum mismatch is a **deterministic access-key validation failure**. It is not, by itself, a statement about whether SEFAZ has authorized the document.

---

## 5. Raw `NFe` vs. `nfeProc`

A raw `<NFe>` contains the invoice payload.

A `<nfeProc>` wrapper can include `<protNFe>` with protocol fields such as:

- `nProt`
- `cStat`
- `xMotivo`
- `dhRecbto`

The proving parser extracts those fields when present.

Important distinction:

```text
protocol data present in XML
        ≠
live SEFAZ query performed now
        ≠
cryptographic signature chain verified
```

Do not describe parser success as “SEFAZ-verified” unless the system actually performs that verification.

---

## 6. Direction Is Contextual

NF-e XML identifies issuer (`emit`) and recipient (`dest`), but “inbound” and “outbound” are relative to the active business.

A useful deterministic rule is:

```text
tenant document == issuer document    → outbound
tenant document == recipient document → inbound
otherwise                              → unknown
```

This is a **PROJECT/DOMAIN CONVENTION** for downstream analytics, not a field asserted by the XML itself.

---

## 7. Totals Consistency Policy

The proving project compares item `vProd` totals against declared invoice totals using exact decimals.

Astra currently applies this policy:

```text
difference == 0      → exact
0 < difference <= .02 → rounding warning
> .02                 → review warning
```

The **R$ 0.02 threshold is an Astra policy**, not a universal NF-e rule encoded by SurrealDB or `rust_decimal`.

If another system has a different regulatory/accounting tolerance, parameterize or replace it deliberately.

Never silently rewrite declared XML values to force totals to match.

---

## 8. Replay vs. Identity Conflict

The proving persistence model distinguishes:

### Idempotent replay

```text
same access key
+ same source artifact identity
→ return existing authoritative record
```

### Identity conflict

```text
same access key
+ different source artifact
→ explicit conflict
```

Astra names the second condition:

```text
NFE_DUPLICATE_CONFLICT
```

The general lesson is to distinguish **replay** from **conflicting evidence** rather than treating both as “duplicate, ignore.”

---

## 9. Structured Source vs. Human/LLM Representation

For structured fiscal XML:

```text
XML typed parser
→ authoritative extracted fields

Markdown/readable representation
→ human/LLM convenience
```

Do not make OCR or Markdown the source of truth for a value that exists deterministically in the XML.

If the two disagree, preserve the original XML and investigate the representation layer.

---

## 10. Persistence Atomicity Is a Separate Concern

A correct parser does not guarantee correct materialization.

If an invoice expands into:

```text
root document
+ parties
+ items
+ taxes
+ totals
+ payments
+ volumes
```

then a failure after writing only some children can leave a partial business object unless persistence is transactionally all-or-nothing or has an equivalent recovery protocol.

As of the current proving baseline, **Astra Phase 5 sequentially writes these records and has not yet proven rollback through failure injection**.

Therefore:

> Do not describe current NF-e multi-table materialization as transactionally atomic.

This is intentionally recorded as an open issue in `verification-status.md`.

---

## 11. Minimum Test Matrix

A deterministic NF-e parser should cover at least:

```text
valid CNPJ issuer + CNPJ recipient
valid CPF recipient
raw NFe
nfeProc
multiple items
multiple tax groups
freight/insurance/discount
payments
transport volumes
exact-decimal edge cases
valid access-key checksum
invalid checksum
malformed XML
unsupported XML root
missing mandatory structure
DTD/entity attack
excessive nesting
replay
same-key conflicting source
persistence restart
```

Use sanitized synthetic fixtures, not customer fiscal documents.

## NF-e Signatures: What the File Can and Cannot Prove

Research and a prototype for a proving repository (Astra, 2026-10-09/10). Sources: the national *Manual de Orientação do Contribuinte* (updated to NT 2026.007) and the official schema package PL_010_V1.30.

What the specification fixes (**VERIFIED API**, from the official schema and manual):

- The issuer signs `infNFe`, referenced as `URI="#NFe<key>"`. `ds:Signature` is the last child of `NFe`, a sibling of `infNFe`, so the enveloped-signature transform removes nothing from the signed content.
- The algorithms are fixed by the schema: inclusive C14N 1.0 (`http://www.w3.org/TR/2001/REC-xml-c14n-20010315`), `rsa-sha1`, `sha1`. A file using anything else is not a valid NF-e. No current technical note moves the issuer signature to SHA-256.
- `KeyInfo` carries only the end-entity certificate. A verifier must bring its own ICP-Brasil intermediates and roots, and must never trust the key in the file by itself.
- A CNPJ certificate must match the first 8 characters of `emit/CNPJ` (any branch may sign); a CPF certificate must match `emit/CPF` exactly. Invoices issued by SEFAZ, NFF and PAA invoices have special signers.
- `protNFe/infProt/digVal` is a copy of the issuer's `DigestValue`, and the protocol's own signature is optional. A typed-up `protNFe` with `cStat 100` on a genuinely signed invoice passes every offline check.
- Cancellation is a separate event and never changes the `nfeProc` file.

Consequences:

1. A valid issuer signature proves the content is what the issuer signed. It does not prove SEFAZ authorized it, nor that it was not cancelled later. Only an online check (for example `NFeConsultaProtocolo4` with an ICP-Brasil certificate) can.
2. Never show "authorized" from the file's own `cStat`. Show "stated in the file" until an authoritative source confirms it.

Implementation traps (**CASE-STUDY EVIDENCE**, observed in a scratch prototype):

- **Comment split.** C14N without comments drops comments, so `<vNF>50<!---->00.00</vNF>` keeps a valid signature, while a reader that takes only the first text node (`roxmltree::Node::text()`, lxml `.text`) reads `50`. Reject comments and processing instructions inside fiscal content, or concatenate all text children, before trusting a value.
- **lxml subtree C14N.** `etree.tostring(element, method="c14n")` on a sub-element added a spurious `xmlns=""` to nested elements that inherit the default namespace (lxml 6.1.3 / libxml2 2.14.6), so real signatures never matched. Canonicalize with a method proven on real signed samples.
- **Re-indented files.** Pretty-printing changes whitespace text nodes inside the signed element and breaks the digest. A single, clearly labelled retry without whitespace-only text between tags is the most a verifier should attempt.
- **Two parsers, two truths.** Verify the signature on the same parsed tree that produces the displayed values, or the comment split reopens in another form.
