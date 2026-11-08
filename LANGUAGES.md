# Supported Languages

> **Auto-generated** — do not edit by hand. Run `cargo run --example generate_languages_md` to update.

n2words supports **50 languages** (72 regional/script variants total) with
cardinal number conversion, 3 variants with ordinal support, 3 variants
with currency support.

46 languages have a **bare-tag entry point** that resolves without a region
subtag (`n2words::de`); the other 4 require a specific variant because their
variants genuinely diverge in script or core numbering grammar.

Language codes follow [IETF BCP 47](https://tools.ietf.org/html/bcp47); the Rust module is the code in snake_case (`en_us`, `zh_hans_cn`).

## Languages

|Entry point|Language|Variants|
|-----------|--------|--------|
|—|Amharic|[`am-ET`](#amharic-ethiopia-am-et), [`am-Latn-ET`](#amharic-latin-ethiopia-am-latn-et)|
|[`ar`](#arabic-saudi-arabia-ar-sa)|Arabic|[`ar-SA`](#arabic-saudi-arabia-ar-sa)|
|[`az`](#azerbaijani-azerbaijan-az-az)|Azerbaijani|[`az-AZ`](#azerbaijani-azerbaijan-az-az)|
|[`bn`](#bangla-bangladesh-bn-bd)|Bangla|[`bn-BD`](#bangla-bangladesh-bn-bd)|
|[`hbo`](#biblical-hebrew-israel-hbo-il)|Biblical Hebrew|[`hbo-IL`](#biblical-hebrew-israel-hbo-il)|
|—|Chinese|[`zh-Hans-CN`](#chinese-simplified-china-zh-hans-cn), [`zh-Hant-TW`](#chinese-traditional-taiwan-zh-hant-tw)|
|[`hr`](#croatian-croatia-hr-hr)|Croatian|[`hr-HR`](#croatian-croatia-hr-hr)|
|[`cs`](#czech-czechia-cs-cz)|Czech|[`cs-CZ`](#czech-czechia-cs-cz)|
|[`da`](#danish-denmark-da-dk)|Danish|[`da-DK`](#danish-denmark-da-dk)|
|[`nl`](#dutch-netherlands-nl-nl)|Dutch|[`nl-NL`](#dutch-netherlands-nl-nl)|
|[`en`](#american-english-en-us)|English|[`en-AU`](#australian-english-en-au), [`en-BD`](#english-bangladesh-en-bd), [`en-CA`](#canadian-english-en-ca), [`en-GB`](#british-english-en-gb), [`en-GH`](#english-ghana-en-gh), [`en-IE`](#english-ireland-en-ie), [`en-IN`](#english-india-en-in), [`en-KE`](#english-kenya-en-ke), [`en-MY`](#english-malaysia-en-my), [`en-NG`](#english-nigeria-en-ng), [`en-NZ`](#english-new-zealand-en-nz), [`en-PH`](#english-philippines-en-ph), [`en-PK`](#english-pakistan-en-pk), [`en-SG`](#english-singapore-en-sg), [`en-US`](#american-english-en-us) (default), [`en-ZA`](#english-south-africa-en-za)|
|[`fil`](#filipino-philippines-fil-ph)|Filipino|[`fil-PH`](#filipino-philippines-fil-ph)|
|[`fi`](#finnish-finland-fi-fi)|Finnish|[`fi-FI`](#finnish-finland-fi-fi)|
|[`fr`](#french-france-fr-fr)|French|[`fr-BE`](#french-belgium-fr-be), [`fr-FR`](#french-france-fr-fr) (default)|
|[`ka`](#georgian-georgia-ka-ge)|Georgian|[`ka-GE`](#georgian-georgia-ka-ge)|
|[`de`](#german-germany-de-de)|German|[`de-DE`](#german-germany-de-de)|
|[`el`](#greek-greece-el-gr)|Greek|[`el-GR`](#greek-greece-el-gr)|
|[`gu`](#gujarati-india-gu-in)|Gujarati|[`gu-IN`](#gujarati-india-gu-in)|
|[`ha`](#hausa-nigeria-ha-ng)|Hausa|[`ha-NG`](#hausa-nigeria-ha-ng)|
|[`he`](#hebrew-israel-he-il)|Hebrew|[`he-IL`](#hebrew-israel-he-il)|
|[`hi`](#hindi-india-hi-in)|Hindi|[`hi-IN`](#hindi-india-hi-in)|
|[`hu`](#hungarian-hungary-hu-hu)|Hungarian|[`hu-HU`](#hungarian-hungary-hu-hu)|
|[`id`](#indonesian-indonesia-id-id)|Indonesian|[`id-ID`](#indonesian-indonesia-id-id)|
|[`it`](#italian-italy-it-it)|Italian|[`it-IT`](#italian-italy-it-it)|
|[`ja`](#japanese-japan-ja-jp)|Japanese|[`ja-JP`](#japanese-japan-ja-jp)|
|[`kn`](#kannada-india-kn-in)|Kannada|[`kn-IN`](#kannada-india-kn-in)|
|[`ko`](#korean-south-korea-ko-kr)|Korean|[`ko-KR`](#korean-south-korea-ko-kr)|
|[`lv`](#latvian-latvia-lv-lv)|Latvian|[`lv-LV`](#latvian-latvia-lv-lv)|
|[`lt`](#lithuanian-lithuania-lt-lt)|Lithuanian|[`lt-LT`](#lithuanian-lithuania-lt-lt)|
|[`ms`](#malay-malaysia-ms-my)|Malay|[`ms-MY`](#malay-malaysia-ms-my)|
|[`mr`](#marathi-india-mr-in)|Marathi|[`mr-IN`](#marathi-india-mr-in)|
|[`nb`](#norwegian-bokmål-norway-nb-no)|Norwegian Bokmål|[`nb-NO`](#norwegian-bokmål-norway-nb-no)|
|[`fa`](#persian-iran-fa-ir)|Persian|[`fa-IR`](#persian-iran-fa-ir)|
|[`pl`](#polish-poland-pl-pl)|Polish|[`pl-PL`](#polish-poland-pl-pl)|
|—|Portuguese|[`pt-BR`](#brazilian-portuguese-pt-br), [`pt-PT`](#european-portuguese-pt-pt)|
|[`pa`](#punjabi-india-pa-in)|Punjabi|[`pa-IN`](#punjabi-india-pa-in)|
|[`ro`](#romanian-romania-ro-ro)|Romanian|[`ro-RO`](#romanian-romania-ro-ro)|
|[`ru`](#russian-russia-ru-ru)|Russian|[`ru-RU`](#russian-russia-ru-ru)|
|—|Serbian|[`sr-Cyrl-RS`](#serbian-cyrillic-serbia-sr-cyrl-rs), [`sr-Latn-RS`](#serbian-latin-serbia-sr-latn-rs)|
|[`es`](#european-spanish-es-es)|Spanish|[`es-ES`](#european-spanish-es-es) (default), [`es-MX`](#mexican-spanish-es-mx), [`es-US`](#spanish-united-states-es-us)|
|[`sw`](#swahili-kenya-sw-ke)|Swahili|[`sw-KE`](#swahili-kenya-sw-ke)|
|[`sv`](#swedish-sweden-sv-se)|Swedish|[`sv-SE`](#swedish-sweden-sv-se)|
|[`ta`](#tamil-india-ta-in)|Tamil|[`ta-IN`](#tamil-india-ta-in)|
|[`te`](#telugu-india-te-in)|Telugu|[`te-IN`](#telugu-india-te-in)|
|[`th`](#thai-thailand-th-th)|Thai|[`th-TH`](#thai-thailand-th-th)|
|[`tr`](#turkish-türkiye-tr-tr)|Turkish|[`tr-TR`](#turkish-türkiye-tr-tr)|
|[`uk`](#ukrainian-ukraine-uk-ua)|Ukrainian|[`uk-UA`](#ukrainian-ukraine-uk-ua)|
|[`ur`](#urdu-pakistan-ur-pk)|Urdu|[`ur-PK`](#urdu-pakistan-ur-pk)|
|[`vi`](#vietnamese-vietnam-vi-vn)|Vietnamese|[`vi-VN`](#vietnamese-vietnam-vi-vn)|
|[`yo`](#yoruba-nigeria-yo-ng)|Yoruba|[`yo-NG`](#yoruba-nigeria-yo-ng)|

`Entry point` is `—` for the languages with no safe default variant. Where a
family has more than one variant, `(default)` marks the one its entry point
resolves to.

## Usage

```rust
use n2words::{de, en_gb, Currency};

de::to_cardinal(42)?;                 // "zweiundvierzig"
de::to_ordinal(42)?;                  // "zweiundvierzigste"
de::to_currency(42.5, Currency::EUR)?; // a bare tag names the currency explicitly
en_gb::to_currency(42.5)?;            // a region-qualified code has a default
```

## All Regional Variants

Per-variant detail: the largest value each form converts, with a `*` where the
variant declares options (click through to the options table).

|Code|Language|Cardinal|Ordinal|Currency|
|----|--------|:------:|:-----:|:------:|
|`am-ET`|Amharic (Ethiopia)||||
|`am-Latn-ET`|Amharic (Latin, Ethiopia)||||
|`ar-SA`|Arabic (Saudi Arabia)||||
|`az-AZ`|Azerbaijani (Azerbaijan)||||
|`bn-BD`|Bangla (Bangladesh)||||
|`cs-CZ`|Czech (Czechia)||||
|`da-DK`|Danish (Denmark)||||
|`de-DE`|German (Germany)||||
|`el-GR`|Greek (Greece)||||
|`en-AU`|Australian English|10^66 - 1|10^66 - 1|10^66 - 1 [*](#australian-english-en-au)|
|`en-BD`|English (Bangladesh)||||
|`en-CA`|Canadian English||||
|`en-GB`|British English|10^66 - 1|10^66 - 1|10^66 - 1 [*](#british-english-en-gb)|
|`en-GH`|English (Ghana)||||
|`en-IE`|English (Ireland)||||
|`en-IN`|English (India)||||
|`en-KE`|English (Kenya)||||
|`en-MY`|English (Malaysia)||||
|`en-NG`|English (Nigeria)||||
|`en-NZ`|English (New Zealand)||||
|`en-PH`|English (Philippines)||||
|`en-PK`|English (Pakistan)||||
|`en-SG`|English (Singapore)||||
|`en-US`|American English|10^66 - 1 [*](#american-english-en-us)|10^66 - 1|10^66 - 1 [*](#american-english-en-us)|
|`en-ZA`|English (South Africa)||||
|`es-ES`|European Spanish||||
|`es-MX`|Mexican Spanish||||
|`es-US`|Spanish (United States)||||
|`fa-IR`|Persian (Iran)||||
|`fi-FI`|Finnish (Finland)||||
|`fil-PH`|Filipino (Philippines)||||
|`fr-BE`|French (Belgium)||||
|`fr-FR`|French (France)||||
|`gu-IN`|Gujarati (India)||||
|`ha-NG`|Hausa (Nigeria)||||
|`hbo-IL`|Biblical Hebrew (Israel)||||
|`he-IL`|Hebrew (Israel)||||
|`hi-IN`|Hindi (India)||||
|`hr-HR`|Croatian (Croatia)||||
|`hu-HU`|Hungarian (Hungary)||||
|`id-ID`|Indonesian (Indonesia)||||
|`it-IT`|Italian (Italy)||||
|`ja-JP`|Japanese (Japan)||||
|`ka-GE`|Georgian (Georgia)||||
|`kn-IN`|Kannada (India)||||
|`ko-KR`|Korean (South Korea)||||
|`lt-LT`|Lithuanian (Lithuania)||||
|`lv-LV`|Latvian (Latvia)||||
|`mr-IN`|Marathi (India)||||
|`ms-MY`|Malay (Malaysia)||||
|`nb-NO`|Norwegian Bokmål (Norway)||||
|`nl-NL`|Dutch (Netherlands)||||
|`pa-IN`|Punjabi (India)||||
|`pl-PL`|Polish (Poland)||||
|`pt-BR`|Brazilian Portuguese||||
|`pt-PT`|European Portuguese||||
|`ro-RO`|Romanian (Romania)||||
|`ru-RU`|Russian (Russia)||||
|`sr-Cyrl-RS`|Serbian (Cyrillic, Serbia)||||
|`sr-Latn-RS`|Serbian (Latin, Serbia)||||
|`sv-SE`|Swedish (Sweden)||||
|`sw-KE`|Swahili (Kenya)||||
|`ta-IN`|Tamil (India)||||
|`te-IN`|Telugu (India)||||
|`th-TH`|Thai (Thailand)||||
|`tr-TR`|Turkish (Türkiye)||||
|`uk-UA`|Ukrainian (Ukraine)||||
|`ur-PK`|Urdu (Pakistan)||||
|`vi-VN`|Vietnamese (Vietnam)||||
|`yo-NG`|Yoruba (Nigeria)||||
|`zh-Hans-CN`|Chinese (Simplified, China)||||
|`zh-Hant-TW`|Chinese (Traditional, Taiwan)||||

Each form column shows the largest value it converts (`10^N - 1`), `∞` when unbounded, or blank when the form isn't supported.

## Currency Coverage

Which languages can name which currency. 3 variants name 24 distinct
ISO 4217 currencies, across 72 language/currency pairs. A language always
names its own default currency; anything in the third column is opt-in via the
`currency` option, and asking for a currency a language has no words for is an
`Error::UnsupportedCurrency`, never a guess.

|Currency|Default for|Also names it|
|--------|-----------|-------------|
|`AUD`|`en-AU`|`en`|
|`BDT`|—|`en`|
|`BHD`|—|`en`|
|`CAD`|—|`en`|
|`EUR`|—|`en`|
|`GBP`|`en-GB`|`en`|
|`GHS`|—|`en`|
|`INR`|—|`en`|
|`IQD`|—|`en`|
|`JOD`|—|`en`|
|`KES`|—|`en`|
|`KWD`|—|`en`|
|`LYD`|—|`en`|
|`MAD`|—|`en`|
|`MYR`|—|`en`|
|`NGN`|—|`en`|
|`NZD`|—|`en`|
|`OMR`|—|`en`|
|`PHP`|—|`en`|
|`PKR`|—|`en`|
|`SGD`|—|`en`|
|`TND`|—|`en`|
|`USD`|`en-US`|`en`|
|`ZAR`|—|`en`|

## Language Options

3 variants take options through the `to_<form>_with` functions and their
`<Form>Options` structs (whose `Default` is the language's defaults). Every option
belongs to exactly one form. In the dynamic API and the CLI the same options are
addressed by name (`--gender feminine`, `--no-and`).

### American English (`en-US`)

|Option|Form|Type|Default|Description|
|------|----|----|-------|-----------|
|`hundred_pairing`|cardinal|`boolean`|`false`|Use hundred-pairing for 1100-9999 (e.g., "fifteen hundred" instead of "one thousand five hundred")|
|`and`|cardinal|`boolean`|`false`|Use "and" after hundreds and before final small numbers (e.g., "one hundred and one" instead of "one hundred one")|
|`and`|currency|`boolean`|`true`|Use "and" between dollars and cents (e.g., "one dollar and fifty cents")|
|`currency`|currency|'USD' \| 'CAD' \| 'AUD' \| 'NZD' \| 'SGD' \| 'GBP' \| 'EUR' \| 'BDT' \| 'GHS' \| 'INR' \| 'KES' \| 'MYR' \| 'NGN' \| 'PHP' \| 'PKR' \| 'ZAR' \| 'TND' \| 'KWD' \| 'BHD' \| 'JOD' \| 'IQD' \| 'OMR' \| 'LYD' \| 'MAD'|`USD`|ISO 4217 currency code to name the amount in|

### Australian English (`en-AU`)

|Option|Form|Type|Default|Description|
|------|----|----|-------|-----------|
|`and`|currency|`boolean`|`true`|Use "and" between the major and minor unit|
|`currency`|currency|'USD' \| 'CAD' \| 'AUD' \| 'NZD' \| 'SGD' \| 'GBP' \| 'EUR' \| 'BDT' \| 'GHS' \| 'INR' \| 'KES' \| 'MYR' \| 'NGN' \| 'PHP' \| 'PKR' \| 'ZAR' \| 'TND' \| 'KWD' \| 'BHD' \| 'JOD' \| 'IQD' \| 'OMR' \| 'LYD' \| 'MAD'|`AUD`|ISO 4217 currency code to name the amount in|

### British English (`en-GB`)

|Option|Form|Type|Default|Description|
|------|----|----|-------|-----------|
|`and`|currency|`boolean`|`true`|Use "and" between pounds and pence (e.g., "one pound and fifty pence")|
|`currency`|currency|'USD' \| 'CAD' \| 'AUD' \| 'NZD' \| 'SGD' \| 'GBP' \| 'EUR' \| 'BDT' \| 'GHS' \| 'INR' \| 'KES' \| 'MYR' \| 'NGN' \| 'PHP' \| 'PKR' \| 'ZAR' \| 'TND' \| 'KWD' \| 'BHD' \| 'JOD' \| 'IQD' \| 'OMR' \| 'LYD' \| 'MAD'|`GBP`|ISO 4217 currency code to name the amount in|
