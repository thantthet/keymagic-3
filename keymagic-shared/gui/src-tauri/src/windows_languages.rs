// Windows Language Definitions based on MS-LCID
// Source: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-lcid/63d3d639-7fd2-4afb-abbe-0d5b5551eef8

use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WindowsLanguage {
    pub lcid: u16,
    pub code: &'static str,
    pub name: &'static str,
}

// Create a static HashMap for quick lookups
pub static WINDOWS_LANGUAGES: Lazy<HashMap<&'static str, WindowsLanguage>> = Lazy::new(|| {
    let languages = vec![
        // Most common languages first
        WindowsLanguage { lcid: 0x0409, code: "en-US", name: "English (United States)" },
        WindowsLanguage { lcid: 0x0809, code: "en-GB", name: "English (United Kingdom)" },
        WindowsLanguage { lcid: 0x0c09, code: "en-AU", name: "English (Australia)" },
        WindowsLanguage { lcid: 0x1009, code: "en-CA", name: "English (Canada)" },
        WindowsLanguage { lcid: 0x1409, code: "en-NZ", name: "English (New Zealand)" },
        WindowsLanguage { lcid: 0x1809, code: "en-IE", name: "English (Ireland)" },
        WindowsLanguage { lcid: 0x1c09, code: "en-ZA", name: "English (South Africa)" },
        WindowsLanguage { lcid: 0x2009, code: "en-JM", name: "English (Jamaica)" },
        WindowsLanguage { lcid: 0x2409, code: "en-029", name: "English (Caribbean)" },
        WindowsLanguage { lcid: 0x2809, code: "en-BZ", name: "English (Belize)" },
        WindowsLanguage { lcid: 0x2c09, code: "en-TT", name: "English (Trinidad and Tobago)" },
        WindowsLanguage { lcid: 0x3009, code: "en-ZW", name: "English (Zimbabwe)" },
        WindowsLanguage { lcid: 0x3409, code: "en-PH", name: "English (Philippines)" },
        WindowsLanguage { lcid: 0x4009, code: "en-IN", name: "English (India)" },
        WindowsLanguage { lcid: 0x4409, code: "en-MY", name: "English (Malaysia)" },
        WindowsLanguage { lcid: 0x4809, code: "en-SG", name: "English (Singapore)" },
        
        // Chinese variants
        WindowsLanguage { lcid: 0x0804, code: "zh-CN", name: "Chinese (Simplified, China)" },
        WindowsLanguage { lcid: 0x0404, code: "zh-TW", name: "Chinese (Traditional, Taiwan)" },
        WindowsLanguage { lcid: 0x0c04, code: "zh-HK", name: "Chinese (Traditional, Hong Kong SAR)" },
        WindowsLanguage { lcid: 0x1004, code: "zh-SG", name: "Chinese (Simplified, Singapore)" },
        WindowsLanguage { lcid: 0x1404, code: "zh-MO", name: "Chinese (Traditional, Macao SAR)" },
        
        // Spanish variants
        WindowsLanguage { lcid: 0x040a, code: "es-ES", name: "Spanish (Spain)" },
        WindowsLanguage { lcid: 0x080a, code: "es-MX", name: "Spanish (Mexico)" },
        WindowsLanguage { lcid: 0x0c0a, code: "es-ES", name: "Spanish (Modern Sort, Spain)" },
        WindowsLanguage { lcid: 0x100a, code: "es-GT", name: "Spanish (Guatemala)" },
        WindowsLanguage { lcid: 0x140a, code: "es-CR", name: "Spanish (Costa Rica)" },
        WindowsLanguage { lcid: 0x180a, code: "es-PA", name: "Spanish (Panama)" },
        WindowsLanguage { lcid: 0x1c0a, code: "es-DO", name: "Spanish (Dominican Republic)" },
        WindowsLanguage { lcid: 0x200a, code: "es-VE", name: "Spanish (Venezuela)" },
        WindowsLanguage { lcid: 0x240a, code: "es-CO", name: "Spanish (Colombia)" },
        WindowsLanguage { lcid: 0x280a, code: "es-PE", name: "Spanish (Peru)" },
        WindowsLanguage { lcid: 0x2c0a, code: "es-AR", name: "Spanish (Argentina)" },
        WindowsLanguage { lcid: 0x300a, code: "es-EC", name: "Spanish (Ecuador)" },
        WindowsLanguage { lcid: 0x340a, code: "es-CL", name: "Spanish (Chile)" },
        WindowsLanguage { lcid: 0x380a, code: "es-UY", name: "Spanish (Uruguay)" },
        WindowsLanguage { lcid: 0x3c0a, code: "es-PY", name: "Spanish (Paraguay)" },
        WindowsLanguage { lcid: 0x400a, code: "es-BO", name: "Spanish (Bolivia)" },
        WindowsLanguage { lcid: 0x440a, code: "es-SV", name: "Spanish (El Salvador)" },
        WindowsLanguage { lcid: 0x480a, code: "es-HN", name: "Spanish (Honduras)" },
        WindowsLanguage { lcid: 0x4c0a, code: "es-NI", name: "Spanish (Nicaragua)" },
        WindowsLanguage { lcid: 0x500a, code: "es-PR", name: "Spanish (Puerto Rico)" },
        WindowsLanguage { lcid: 0x540a, code: "es-US", name: "Spanish (United States)" },
        
        // French variants
        WindowsLanguage { lcid: 0x040c, code: "fr-FR", name: "French (France)" },
        WindowsLanguage { lcid: 0x080c, code: "fr-BE", name: "French (Belgium)" },
        WindowsLanguage { lcid: 0x0c0c, code: "fr-CA", name: "French (Canada)" },
        WindowsLanguage { lcid: 0x100c, code: "fr-CH", name: "French (Switzerland)" },
        WindowsLanguage { lcid: 0x140c, code: "fr-LU", name: "French (Luxembourg)" },
        WindowsLanguage { lcid: 0x180c, code: "fr-MC", name: "French (Monaco)" },
        
        // German variants
        WindowsLanguage { lcid: 0x0407, code: "de-DE", name: "German (Germany)" },
        WindowsLanguage { lcid: 0x0807, code: "de-CH", name: "German (Switzerland)" },
        WindowsLanguage { lcid: 0x0c07, code: "de-AT", name: "German (Austria)" },
        WindowsLanguage { lcid: 0x1007, code: "de-LU", name: "German (Luxembourg)" },
        WindowsLanguage { lcid: 0x1407, code: "de-LI", name: "German (Liechtenstein)" },
        
        // Portuguese variants
        WindowsLanguage { lcid: 0x0416, code: "pt-BR", name: "Portuguese (Brazil)" },
        WindowsLanguage { lcid: 0x0816, code: "pt-PT", name: "Portuguese (Portugal)" },
        
        // Southeast Asian languages
        WindowsLanguage { lcid: 0x0455, code: "my-MM", name: "Myanmar" },
        WindowsLanguage { lcid: 0x041e, code: "th-TH", name: "Thai" },
        WindowsLanguage { lcid: 0x0453, code: "km-KH", name: "Khmer (Cambodia)" },
        WindowsLanguage { lcid: 0x0454, code: "lo-LA", name: "Lao" },
        WindowsLanguage { lcid: 0x042a, code: "vi-VN", name: "Vietnamese" },
        WindowsLanguage { lcid: 0x0421, code: "id-ID", name: "Indonesian" },
        WindowsLanguage { lcid: 0x043e, code: "ms-MY", name: "Malay (Malaysia)" },
        WindowsLanguage { lcid: 0x083e, code: "ms-BN", name: "Malay (Brunei Darussalam)" },
        WindowsLanguage { lcid: 0x0464, code: "fil-PH", name: "Filipino" },
        
        // South Asian languages
        WindowsLanguage { lcid: 0x0439, code: "hi-IN", name: "Hindi" },
        WindowsLanguage { lcid: 0x0445, code: "bn-IN", name: "Bengali (India)" },
        WindowsLanguage { lcid: 0x0845, code: "bn-BD", name: "Bengali (Bangladesh)" },
        WindowsLanguage { lcid: 0x0446, code: "pa-IN", name: "Punjabi (India)" },
        WindowsLanguage { lcid: 0x0447, code: "gu-IN", name: "Gujarati" },
        WindowsLanguage { lcid: 0x0448, code: "or-IN", name: "Odia" },
        WindowsLanguage { lcid: 0x0449, code: "ta-IN", name: "Tamil (India)" },
        WindowsLanguage { lcid: 0x0849, code: "ta-LK", name: "Tamil (Sri Lanka)" },
        WindowsLanguage { lcid: 0x044a, code: "te-IN", name: "Telugu" },
        WindowsLanguage { lcid: 0x044b, code: "kn-IN", name: "Kannada" },
        WindowsLanguage { lcid: 0x044c, code: "ml-IN", name: "Malayalam" },
        WindowsLanguage { lcid: 0x044d, code: "as-IN", name: "Assamese" },
        WindowsLanguage { lcid: 0x044e, code: "mr-IN", name: "Marathi" },
        WindowsLanguage { lcid: 0x044f, code: "sa-IN", name: "Sanskrit" },
        WindowsLanguage { lcid: 0x0457, code: "kok-IN", name: "Konkani" },
        WindowsLanguage { lcid: 0x0461, code: "ne-NP", name: "Nepali (Nepal)" },
        WindowsLanguage { lcid: 0x0861, code: "ne-IN", name: "Nepali (India)" },
        WindowsLanguage { lcid: 0x045b, code: "si-LK", name: "Sinhala" },
        WindowsLanguage { lcid: 0x0463, code: "ps-AF", name: "Pashto" },
        
        // East Asian languages
        WindowsLanguage { lcid: 0x0411, code: "ja-JP", name: "Japanese" },
        WindowsLanguage { lcid: 0x0412, code: "ko-KR", name: "Korean" },
        
        // Middle Eastern languages
        WindowsLanguage { lcid: 0x0401, code: "ar-SA", name: "Arabic (Saudi Arabia)" },
        WindowsLanguage { lcid: 0x0801, code: "ar-IQ", name: "Arabic (Iraq)" },
        WindowsLanguage { lcid: 0x0c01, code: "ar-EG", name: "Arabic (Egypt)" },
        WindowsLanguage { lcid: 0x1001, code: "ar-LY", name: "Arabic (Libya)" },
        WindowsLanguage { lcid: 0x1401, code: "ar-DZ", name: "Arabic (Algeria)" },
        WindowsLanguage { lcid: 0x1801, code: "ar-MA", name: "Arabic (Morocco)" },
        WindowsLanguage { lcid: 0x1c01, code: "ar-TN", name: "Arabic (Tunisia)" },
        WindowsLanguage { lcid: 0x2001, code: "ar-OM", name: "Arabic (Oman)" },
        WindowsLanguage { lcid: 0x2401, code: "ar-YE", name: "Arabic (Yemen)" },
        WindowsLanguage { lcid: 0x2801, code: "ar-SY", name: "Arabic (Syria)" },
        WindowsLanguage { lcid: 0x2c01, code: "ar-JO", name: "Arabic (Jordan)" },
        WindowsLanguage { lcid: 0x3001, code: "ar-LB", name: "Arabic (Lebanon)" },
        WindowsLanguage { lcid: 0x3401, code: "ar-KW", name: "Arabic (Kuwait)" },
        WindowsLanguage { lcid: 0x3801, code: "ar-AE", name: "Arabic (U.A.E.)" },
        WindowsLanguage { lcid: 0x3c01, code: "ar-BH", name: "Arabic (Bahrain)" },
        WindowsLanguage { lcid: 0x4001, code: "ar-QA", name: "Arabic (Qatar)" },
        WindowsLanguage { lcid: 0x040d, code: "he-IL", name: "Hebrew" },
        WindowsLanguage { lcid: 0x0429, code: "fa-IR", name: "Persian" },
        WindowsLanguage { lcid: 0x041f, code: "tr-TR", name: "Turkish" },
        WindowsLanguage { lcid: 0x0422, code: "uk-UA", name: "Ukrainian" },
        WindowsLanguage { lcid: 0x0420, code: "ur-PK", name: "Urdu (Pakistan)" },
        WindowsLanguage { lcid: 0x0820, code: "ur-IN", name: "Urdu (India)" },
        
        // European languages
        WindowsLanguage { lcid: 0x0405, code: "cs-CZ", name: "Czech" },
        WindowsLanguage { lcid: 0x0406, code: "da-DK", name: "Danish" },
        WindowsLanguage { lcid: 0x0408, code: "el-GR", name: "Greek" },
        WindowsLanguage { lcid: 0x040b, code: "fi-FI", name: "Finnish" },
        WindowsLanguage { lcid: 0x040e, code: "hu-HU", name: "Hungarian" },
        WindowsLanguage { lcid: 0x040f, code: "is-IS", name: "Icelandic" },
        WindowsLanguage { lcid: 0x0410, code: "it-IT", name: "Italian (Italy)" },
        WindowsLanguage { lcid: 0x0810, code: "it-CH", name: "Italian (Switzerland)" },
        WindowsLanguage { lcid: 0x0413, code: "nl-NL", name: "Dutch (Netherlands)" },
        WindowsLanguage { lcid: 0x0813, code: "nl-BE", name: "Dutch (Belgium)" },
        WindowsLanguage { lcid: 0x0414, code: "nb-NO", name: "Norwegian (Bokmål)" },
        WindowsLanguage { lcid: 0x0814, code: "nn-NO", name: "Norwegian (Nynorsk)" },
        WindowsLanguage { lcid: 0x0415, code: "pl-PL", name: "Polish" },
        WindowsLanguage { lcid: 0x0418, code: "ro-RO", name: "Romanian" },
        WindowsLanguage { lcid: 0x0419, code: "ru-RU", name: "Russian" },
        WindowsLanguage { lcid: 0x041a, code: "hr-HR", name: "Croatian" },
        WindowsLanguage { lcid: 0x081a, code: "sr-Latn-CS", name: "Serbian (Latin)" },
        WindowsLanguage { lcid: 0x0c1a, code: "sr-Cyrl-CS", name: "Serbian (Cyrillic)" },
        WindowsLanguage { lcid: 0x041b, code: "sk-SK", name: "Slovak" },
        WindowsLanguage { lcid: 0x041c, code: "sq-AL", name: "Albanian" },
        WindowsLanguage { lcid: 0x041d, code: "sv-SE", name: "Swedish (Sweden)" },
        WindowsLanguage { lcid: 0x081d, code: "sv-FI", name: "Swedish (Finland)" },
        WindowsLanguage { lcid: 0x0424, code: "sl-SI", name: "Slovenian" },
        WindowsLanguage { lcid: 0x0425, code: "et-EE", name: "Estonian" },
        WindowsLanguage { lcid: 0x0426, code: "lv-LV", name: "Latvian" },
        WindowsLanguage { lcid: 0x0427, code: "lt-LT", name: "Lithuanian" },
        WindowsLanguage { lcid: 0x042f, code: "mk-MK", name: "Macedonian" },
        WindowsLanguage { lcid: 0x0436, code: "af-ZA", name: "Afrikaans" },
        WindowsLanguage { lcid: 0x0437, code: "ka-GE", name: "Georgian" },
        WindowsLanguage { lcid: 0x0438, code: "fo-FO", name: "Faroese" },
        WindowsLanguage { lcid: 0x043a, code: "mt-MT", name: "Maltese" },
        WindowsLanguage { lcid: 0x043b, code: "se-NO", name: "Sami (Northern, Norway)" },
        WindowsLanguage { lcid: 0x083b, code: "se-SE", name: "Sami (Northern, Sweden)" },
        WindowsLanguage { lcid: 0x0c3b, code: "se-FI", name: "Sami (Northern, Finland)" },
        WindowsLanguage { lcid: 0x103b, code: "smj-NO", name: "Sami (Lule, Norway)" },
        WindowsLanguage { lcid: 0x143b, code: "smj-SE", name: "Sami (Lule, Sweden)" },
        WindowsLanguage { lcid: 0x183b, code: "sma-NO", name: "Sami (Southern, Norway)" },
        WindowsLanguage { lcid: 0x1c3b, code: "sma-SE", name: "Sami (Southern, Sweden)" },
        WindowsLanguage { lcid: 0x203b, code: "sms-FI", name: "Sami (Skolt, Finland)" },
        WindowsLanguage { lcid: 0x243b, code: "smn-FI", name: "Sami (Inari, Finland)" },
        WindowsLanguage { lcid: 0x0441, code: "sw-KE", name: "Swahili" },
        WindowsLanguage { lcid: 0x0442, code: "tk-TM", name: "Turkmen" },
        WindowsLanguage { lcid: 0x0443, code: "uz-Latn-UZ", name: "Uzbek (Latin)" },
        WindowsLanguage { lcid: 0x0843, code: "uz-Cyrl-UZ", name: "Uzbek (Cyrillic)" },
        WindowsLanguage { lcid: 0x0444, code: "tt-RU", name: "Tatar" },
        WindowsLanguage { lcid: 0x0450, code: "mn-MN", name: "Mongolian (Cyrillic)" },
        WindowsLanguage { lcid: 0x0850, code: "mn-Mong-CN", name: "Mongolian (Traditional)" },
        WindowsLanguage { lcid: 0x0451, code: "bo-CN", name: "Tibetan" },
        WindowsLanguage { lcid: 0x0452, code: "cy-GB", name: "Welsh" },
        WindowsLanguage { lcid: 0x0456, code: "gl-ES", name: "Galician" },
        WindowsLanguage { lcid: 0x045a, code: "syr-SY", name: "Syriac" },
        WindowsLanguage { lcid: 0x045d, code: "iu-Cans-CA", name: "Inuktitut (Syllabics)" },
        WindowsLanguage { lcid: 0x085d, code: "iu-Latn-CA", name: "Inuktitut (Latin)" },
        WindowsLanguage { lcid: 0x045e, code: "am-ET", name: "Amharic" },
        WindowsLanguage { lcid: 0x0462, code: "fy-NL", name: "Frisian" },
        WindowsLanguage { lcid: 0x0468, code: "ha-Latn-NG", name: "Hausa" },
        WindowsLanguage { lcid: 0x046a, code: "yo-NG", name: "Yoruba" },
        WindowsLanguage { lcid: 0x046b, code: "quz-BO", name: "Quechua (Bolivia)" },
        WindowsLanguage { lcid: 0x086b, code: "quz-EC", name: "Quechua (Ecuador)" },
        WindowsLanguage { lcid: 0x0c6b, code: "quz-PE", name: "Quechua (Peru)" },
        WindowsLanguage { lcid: 0x046c, code: "nso-ZA", name: "Sesotho sa Leboa" },
        WindowsLanguage { lcid: 0x046d, code: "ba-RU", name: "Bashkir" },
        WindowsLanguage { lcid: 0x046e, code: "lb-LU", name: "Luxembourgish" },
        WindowsLanguage { lcid: 0x046f, code: "kl-GL", name: "Greenlandic" },
        WindowsLanguage { lcid: 0x0470, code: "ig-NG", name: "Igbo" },
        WindowsLanguage { lcid: 0x0478, code: "ii-CN", name: "Yi" },
        WindowsLanguage { lcid: 0x047a, code: "arn-CL", name: "Mapudungun" },
        WindowsLanguage { lcid: 0x047c, code: "moh-CA", name: "Mohawk" },
        WindowsLanguage { lcid: 0x047e, code: "br-FR", name: "Breton" },
        WindowsLanguage { lcid: 0x0480, code: "ug-CN", name: "Uyghur" },
        WindowsLanguage { lcid: 0x0481, code: "mi-NZ", name: "Maori" },
        WindowsLanguage { lcid: 0x0482, code: "oc-FR", name: "Occitan" },
        WindowsLanguage { lcid: 0x0483, code: "co-FR", name: "Corsican" },
        WindowsLanguage { lcid: 0x0484, code: "gsw-FR", name: "Alsatian" },
        WindowsLanguage { lcid: 0x0485, code: "sah-RU", name: "Sakha" },
        WindowsLanguage { lcid: 0x0486, code: "qut-GT", name: "K'iche'" },
        WindowsLanguage { lcid: 0x0487, code: "rw-RW", name: "Kinyarwanda" },
        WindowsLanguage { lcid: 0x0488, code: "wo-SN", name: "Wolof" },
        WindowsLanguage { lcid: 0x048c, code: "prs-AF", name: "Dari" },
        WindowsLanguage { lcid: 0x0491, code: "gd-GB", name: "Scottish Gaelic" },
        
        // African languages
        WindowsLanguage { lcid: 0x0432, code: "tn-ZA", name: "Tswana (South Africa)" },
        WindowsLanguage { lcid: 0x0832, code: "tn-BW", name: "Tswana (Botswana)" },
        WindowsLanguage { lcid: 0x0434, code: "xh-ZA", name: "Xhosa" },
        WindowsLanguage { lcid: 0x0435, code: "zu-ZA", name: "Zulu" },
        
        // Other languages
        WindowsLanguage { lcid: 0x042b, code: "hy-AM", name: "Armenian" },
        WindowsLanguage { lcid: 0x042c, code: "az-Latn-AZ", name: "Azeri (Latin)" },
        WindowsLanguage { lcid: 0x082c, code: "az-Cyrl-AZ", name: "Azeri (Cyrillic)" },
        WindowsLanguage { lcid: 0x042d, code: "eu-ES", name: "Basque" },
        WindowsLanguage { lcid: 0x0423, code: "be-BY", name: "Belarusian" },
        WindowsLanguage { lcid: 0x0402, code: "bg-BG", name: "Bulgarian" },
        WindowsLanguage { lcid: 0x0403, code: "ca-ES", name: "Catalan" },
        WindowsLanguage { lcid: 0x0428, code: "tg-Cyrl-TJ", name: "Tajik" },
        WindowsLanguage { lcid: 0x0440, code: "ky-KG", name: "Kyrgyz" },
        WindowsLanguage { lcid: 0x042e, code: "hsb-DE", name: "Upper Sorbian" },
        WindowsLanguage { lcid: 0x082e, code: "dsb-DE", name: "Lower Sorbian" },
    ];
    
    languages.into_iter()
        .map(|lang| (lang.code, lang))
        .collect()
});

// Get all languages sorted by name
pub fn get_all_languages() -> Vec<(String, String)> {
    let mut languages: Vec<_> = WINDOWS_LANGUAGES.values()
        .map(|lang| (lang.code.to_string(), lang.name.to_string()))
        .collect();
    
    // Sort by name for easier navigation
    languages.sort_by(|a, b| a.1.cmp(&b.1));
    
    languages
}

// Convert language code to LCID
pub fn language_code_to_lcid(code: &str) -> Option<u16> {
    WINDOWS_LANGUAGES.get(code).map(|lang| lang.lcid)
}

// Search languages by name (case-insensitive)
pub fn search_languages(query: &str) -> Vec<(String, String)> {
    let query = query.to_lowercase();
    
    let mut results: Vec<_> = WINDOWS_LANGUAGES.values()
        .filter(|lang| lang.name.to_lowercase().contains(&query) || 
                       lang.code.to_lowercase().contains(&query))
        .map(|lang| (lang.code.to_string(), lang.name.to_string()))
        .collect();
    
    results.sort_by(|a, b| a.1.cmp(&b.1));
    
    results
}