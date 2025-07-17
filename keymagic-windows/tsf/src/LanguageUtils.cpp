#include "LanguageUtils.h"
#include <unordered_map>

// Language structure to hold language information
struct LanguageInfo {
    LANGID lcid;
    const wchar_t* name;
};

// Static map of language codes to language information
static const std::unordered_map<std::wstring, LanguageInfo> g_LanguageMap = {
    // English variants
    {L"en-US", {0x0409, L"English (United States)"}},
    {L"en-GB", {0x0809, L"English (United Kingdom)"}},
    {L"en-AU", {0x0c09, L"English (Australia)"}},
    {L"en-CA", {0x1009, L"English (Canada)"}},
    {L"en-NZ", {0x1409, L"English (New Zealand)"}},
    {L"en-IE", {0x1809, L"English (Ireland)"}},
    {L"en-ZA", {0x1c09, L"English (South Africa)"}},
    {L"en-JM", {0x2009, L"English (Jamaica)"}},
    {L"en-029", {0x2409, L"English (Caribbean)"}},
    {L"en-BZ", {0x2809, L"English (Belize)"}},
    {L"en-TT", {0x2c09, L"English (Trinidad and Tobago)"}},
    {L"en-ZW", {0x3009, L"English (Zimbabwe)"}},
    {L"en-PH", {0x3409, L"English (Philippines)"}},
    {L"en-IN", {0x4009, L"English (India)"}},
    {L"en-MY", {0x4409, L"English (Malaysia)"}},
    {L"en-SG", {0x4809, L"English (Singapore)"}},
    
    // Chinese variants
    {L"zh-CN", {0x0804, L"Chinese (Simplified, China)"}},
    {L"zh-TW", {0x0404, L"Chinese (Traditional, Taiwan)"}},
    {L"zh-HK", {0x0c04, L"Chinese (Traditional, Hong Kong SAR)"}},
    {L"zh-SG", {0x1004, L"Chinese (Simplified, Singapore)"}},
    {L"zh-MO", {0x1404, L"Chinese (Traditional, Macao SAR)"}},
    
    // Spanish variants
    {L"es-ES", {0x040a, L"Spanish (Spain)"}},
    {L"es-MX", {0x080a, L"Spanish (Mexico)"}},
    {L"es-GT", {0x100a, L"Spanish (Guatemala)"}},
    {L"es-CR", {0x140a, L"Spanish (Costa Rica)"}},
    {L"es-PA", {0x180a, L"Spanish (Panama)"}},
    {L"es-DO", {0x1c0a, L"Spanish (Dominican Republic)"}},
    {L"es-VE", {0x200a, L"Spanish (Venezuela)"}},
    {L"es-CO", {0x240a, L"Spanish (Colombia)"}},
    {L"es-PE", {0x280a, L"Spanish (Peru)"}},
    {L"es-AR", {0x2c0a, L"Spanish (Argentina)"}},
    {L"es-EC", {0x300a, L"Spanish (Ecuador)"}},
    {L"es-CL", {0x340a, L"Spanish (Chile)"}},
    {L"es-UY", {0x380a, L"Spanish (Uruguay)"}},
    {L"es-PY", {0x3c0a, L"Spanish (Paraguay)"}},
    {L"es-BO", {0x400a, L"Spanish (Bolivia)"}},
    {L"es-SV", {0x440a, L"Spanish (El Salvador)"}},
    {L"es-HN", {0x480a, L"Spanish (Honduras)"}},
    {L"es-NI", {0x4c0a, L"Spanish (Nicaragua)"}},
    {L"es-PR", {0x500a, L"Spanish (Puerto Rico)"}},
    {L"es-US", {0x540a, L"Spanish (United States)"}},
    
    // French variants
    {L"fr-FR", {0x040c, L"French (France)"}},
    {L"fr-BE", {0x080c, L"French (Belgium)"}},
    {L"fr-CA", {0x0c0c, L"French (Canada)"}},
    {L"fr-CH", {0x100c, L"French (Switzerland)"}},
    {L"fr-LU", {0x140c, L"French (Luxembourg)"}},
    {L"fr-MC", {0x180c, L"French (Monaco)"}},
    
    // German variants
    {L"de-DE", {0x0407, L"German (Germany)"}},
    {L"de-CH", {0x0807, L"German (Switzerland)"}},
    {L"de-AT", {0x0c07, L"German (Austria)"}},
    {L"de-LU", {0x1007, L"German (Luxembourg)"}},
    {L"de-LI", {0x1407, L"German (Liechtenstein)"}},
    
    // Portuguese variants
    {L"pt-BR", {0x0416, L"Portuguese (Brazil)"}},
    {L"pt-PT", {0x0816, L"Portuguese (Portugal)"}},
    
    // Southeast Asian languages
    {L"my-MM", {0x0455, L"Myanmar"}},
    {L"th-TH", {0x041e, L"Thai"}},
    {L"km-KH", {0x0453, L"Khmer (Cambodia)"}},
    {L"lo-LA", {0x0454, L"Lao"}},
    {L"vi-VN", {0x042a, L"Vietnamese"}},
    {L"id-ID", {0x0421, L"Indonesian"}},
    {L"ms-MY", {0x043e, L"Malay (Malaysia)"}},
    {L"ms-BN", {0x083e, L"Malay (Brunei Darussalam)"}},
    {L"fil-PH", {0x0464, L"Filipino"}},
    
    // South Asian languages
    {L"hi-IN", {0x0439, L"Hindi"}},
    {L"bn-IN", {0x0445, L"Bengali (India)"}},
    {L"bn-BD", {0x0845, L"Bengali (Bangladesh)"}},
    {L"pa-IN", {0x0446, L"Punjabi (India)"}},
    {L"gu-IN", {0x0447, L"Gujarati"}},
    {L"or-IN", {0x0448, L"Odia"}},
    {L"ta-IN", {0x0449, L"Tamil (India)"}},
    {L"ta-LK", {0x0849, L"Tamil (Sri Lanka)"}},
    {L"te-IN", {0x044a, L"Telugu"}},
    {L"kn-IN", {0x044b, L"Kannada"}},
    {L"ml-IN", {0x044c, L"Malayalam"}},
    {L"as-IN", {0x044d, L"Assamese"}},
    {L"mr-IN", {0x044e, L"Marathi"}},
    {L"sa-IN", {0x044f, L"Sanskrit"}},
    {L"kok-IN", {0x0457, L"Konkani"}},
    {L"ne-NP", {0x0461, L"Nepali (Nepal)"}},
    {L"ne-IN", {0x0861, L"Nepali (India)"}},
    {L"si-LK", {0x045b, L"Sinhala"}},
    {L"ps-AF", {0x0463, L"Pashto"}},
    
    // East Asian languages
    {L"ja-JP", {0x0411, L"Japanese"}},
    {L"ko-KR", {0x0412, L"Korean"}},
    
    // Middle Eastern languages
    {L"ar-SA", {0x0401, L"Arabic (Saudi Arabia)"}},
    {L"ar-IQ", {0x0801, L"Arabic (Iraq)"}},
    {L"ar-EG", {0x0c01, L"Arabic (Egypt)"}},
    {L"ar-LY", {0x1001, L"Arabic (Libya)"}},
    {L"ar-DZ", {0x1401, L"Arabic (Algeria)"}},
    {L"ar-MA", {0x1801, L"Arabic (Morocco)"}},
    {L"ar-TN", {0x1c01, L"Arabic (Tunisia)"}},
    {L"ar-OM", {0x2001, L"Arabic (Oman)"}},
    {L"ar-YE", {0x2401, L"Arabic (Yemen)"}},
    {L"ar-SY", {0x2801, L"Arabic (Syria)"}},
    {L"ar-JO", {0x2c01, L"Arabic (Jordan)"}},
    {L"ar-LB", {0x3001, L"Arabic (Lebanon)"}},
    {L"ar-KW", {0x3401, L"Arabic (Kuwait)"}},
    {L"ar-AE", {0x3801, L"Arabic (U.A.E.)"}},
    {L"ar-BH", {0x3c01, L"Arabic (Bahrain)"}},
    {L"ar-QA", {0x4001, L"Arabic (Qatar)"}},
    {L"he-IL", {0x040d, L"Hebrew"}},
    {L"fa-IR", {0x0429, L"Persian"}},
    {L"tr-TR", {0x041f, L"Turkish"}},
    {L"uk-UA", {0x0422, L"Ukrainian"}},
    {L"ur-PK", {0x0420, L"Urdu (Pakistan)"}},
    {L"ur-IN", {0x0820, L"Urdu (India)"}},
    
    // European languages
    {L"cs-CZ", {0x0405, L"Czech"}},
    {L"da-DK", {0x0406, L"Danish"}},
    {L"el-GR", {0x0408, L"Greek"}},
    {L"fi-FI", {0x040b, L"Finnish"}},
    {L"hu-HU", {0x040e, L"Hungarian"}},
    {L"is-IS", {0x040f, L"Icelandic"}},
    {L"it-IT", {0x0410, L"Italian (Italy)"}},
    {L"it-CH", {0x0810, L"Italian (Switzerland)"}},
    {L"nl-NL", {0x0413, L"Dutch (Netherlands)"}},
    {L"nl-BE", {0x0813, L"Dutch (Belgium)"}},
    {L"nb-NO", {0x0414, L"Norwegian (Bokmål)"}},
    {L"nn-NO", {0x0814, L"Norwegian (Nynorsk)"}},
    {L"pl-PL", {0x0415, L"Polish"}},
    {L"ro-RO", {0x0418, L"Romanian"}},
    {L"ru-RU", {0x0419, L"Russian"}},
    {L"hr-HR", {0x041a, L"Croatian"}},
    {L"sr-Latn-CS", {0x081a, L"Serbian (Latin)"}},
    {L"sr-Cyrl-CS", {0x0c1a, L"Serbian (Cyrillic)"}},
    {L"sk-SK", {0x041b, L"Slovak"}},
    {L"sq-AL", {0x041c, L"Albanian"}},
    {L"sv-SE", {0x041d, L"Swedish (Sweden)"}},
    {L"sv-FI", {0x081d, L"Swedish (Finland)"}},
    {L"sl-SI", {0x0424, L"Slovenian"}},
    {L"et-EE", {0x0425, L"Estonian"}},
    {L"lv-LV", {0x0426, L"Latvian"}},
    {L"lt-LT", {0x0427, L"Lithuanian"}},
    {L"mk-MK", {0x042f, L"Macedonian"}},
    {L"af-ZA", {0x0436, L"Afrikaans"}},
    {L"ka-GE", {0x0437, L"Georgian"}},
    {L"fo-FO", {0x0438, L"Faroese"}},
    {L"mt-MT", {0x043a, L"Maltese"}},
    {L"se-NO", {0x043b, L"Sami (Northern, Norway)"}},
    {L"se-SE", {0x083b, L"Sami (Northern, Sweden)"}},
    {L"se-FI", {0x0c3b, L"Sami (Northern, Finland)"}},
    {L"smj-NO", {0x103b, L"Sami (Lule, Norway)"}},
    {L"smj-SE", {0x143b, L"Sami (Lule, Sweden)"}},
    {L"sma-NO", {0x183b, L"Sami (Southern, Norway)"}},
    {L"sma-SE", {0x1c3b, L"Sami (Southern, Sweden)"}},
    {L"sms-FI", {0x203b, L"Sami (Skolt, Finland)"}},
    {L"smn-FI", {0x243b, L"Sami (Inari, Finland)"}},
    {L"sw-KE", {0x0441, L"Swahili"}},
    {L"tk-TM", {0x0442, L"Turkmen"}},
    {L"uz-Latn-UZ", {0x0443, L"Uzbek (Latin)"}},
    {L"uz-Cyrl-UZ", {0x0843, L"Uzbek (Cyrillic)"}},
    {L"tt-RU", {0x0444, L"Tatar"}},
    {L"mn-MN", {0x0450, L"Mongolian (Cyrillic)"}},
    {L"mn-Mong-CN", {0x0850, L"Mongolian (Traditional)"}},
    {L"bo-CN", {0x0451, L"Tibetan"}},
    {L"cy-GB", {0x0452, L"Welsh"}},
    {L"gl-ES", {0x0456, L"Galician"}},
    {L"syr-SY", {0x045a, L"Syriac"}},
    {L"iu-Cans-CA", {0x045d, L"Inuktitut (Syllabics)"}},
    {L"iu-Latn-CA", {0x085d, L"Inuktitut (Latin)"}},
    {L"am-ET", {0x045e, L"Amharic"}},
    {L"fy-NL", {0x0462, L"Frisian"}},
    {L"ha-Latn-NG", {0x0468, L"Hausa"}},
    {L"yo-NG", {0x046a, L"Yoruba"}},
    {L"quz-BO", {0x046b, L"Quechua (Bolivia)"}},
    {L"quz-EC", {0x086b, L"Quechua (Ecuador)"}},
    {L"quz-PE", {0x0c6b, L"Quechua (Peru)"}},
    {L"nso-ZA", {0x046c, L"Sesotho sa Leboa"}},
    {L"ba-RU", {0x046d, L"Bashkir"}},
    {L"lb-LU", {0x046e, L"Luxembourgish"}},
    {L"kl-GL", {0x046f, L"Greenlandic"}},
    {L"ig-NG", {0x0470, L"Igbo"}},
    {L"ii-CN", {0x0478, L"Yi"}},
    {L"arn-CL", {0x047a, L"Mapudungun"}},
    {L"moh-CA", {0x047c, L"Mohawk"}},
    {L"br-FR", {0x047e, L"Breton"}},
    {L"ug-CN", {0x0480, L"Uyghur"}},
    {L"mi-NZ", {0x0481, L"Maori"}},
    {L"oc-FR", {0x0482, L"Occitan"}},
    {L"co-FR", {0x0483, L"Corsican"}},
    {L"gsw-FR", {0x0484, L"Alsatian"}},
    {L"sah-RU", {0x0485, L"Sakha"}},
    {L"qut-GT", {0x0486, L"K'iche'"}},
    {L"rw-RW", {0x0487, L"Kinyarwanda"}},
    {L"wo-SN", {0x0488, L"Wolof"}},
    {L"prs-AF", {0x048c, L"Dari"}},
    {L"gd-GB", {0x0491, L"Scottish Gaelic"}},
    
    // African languages
    {L"tn-ZA", {0x0432, L"Tswana (South Africa)"}},
    {L"tn-BW", {0x0832, L"Tswana (Botswana)"}},
    {L"xh-ZA", {0x0434, L"Xhosa"}},
    {L"zu-ZA", {0x0435, L"Zulu"}},
    
    // Other languages
    {L"hy-AM", {0x042b, L"Armenian"}},
    {L"az-Latn-AZ", {0x042c, L"Azeri (Latin)"}},
    {L"az-Cyrl-AZ", {0x082c, L"Azeri (Cyrillic)"}},
    {L"eu-ES", {0x042d, L"Basque"}},
    {L"be-BY", {0x0423, L"Belarusian"}},
    {L"bg-BG", {0x0402, L"Bulgarian"}},
    {L"ca-ES", {0x0403, L"Catalan"}},
    {L"tg-Cyrl-TJ", {0x0428, L"Tajik"}},
    {L"ky-KG", {0x0440, L"Kyrgyz"}},
    {L"hsb-DE", {0x042e, L"Upper Sorbian"}},
    {L"dsb-DE", {0x082e, L"Lower Sorbian"}}
};

LANGID LanguageCodeToLangId(const std::wstring& languageCode)
{
    auto it = g_LanguageMap.find(languageCode);
    if (it != g_LanguageMap.end()) {
        return it->second.lcid;
    }
    return 0; // Invalid
}

std::wstring GetLanguageName(const std::wstring& languageCode)
{
    auto it = g_LanguageMap.find(languageCode);
    if (it != g_LanguageMap.end()) {
        return it->second.name;
    }
    return L""; // Empty string for invalid code
}

std::vector<std::wstring> GetSupportedLanguageCodes()
{
    std::vector<std::wstring> codes;
    codes.reserve(g_LanguageMap.size());
    
    for (const auto& pair : g_LanguageMap) {
        codes.push_back(pair.first);
    }
    
    return codes;
}