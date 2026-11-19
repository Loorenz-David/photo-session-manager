//! W2·U1: the shipped writing guide (`master_plan_writer.md` §9; book 08C §7; evidence
//! `publication_writer_research.md` §7, §8): the English and Swedish instructions, the title
//! formulas, the dimensions lines and the style examples. The owner edits the instruction and
//! the examples in Settings → Writer (W4); `None` there means these texts.

use crate::writer::config::Example;

/// §9.1, amended by A23 (2026-11-19): the grade word, the stability sentence, the dimensions
/// line and the short paragraphs moved to the four switches of [`crate::writer::config::WriterSettings`].
pub const INSTRUCTION_EN: &str = concat!(
    "You write product texts for a small Swedish shop that sells restored Scandinavian ",
    "mid-century furniture. You receive the item's fields, a short note of what the photos show, ",
    "the titles of matching listings from other sellers, and the owner's earlier answers. Write ",
    "the texts listed in `texts`: a title, a description, a condition report, a search title and ",
    "a search description.",
    "\n\n",
    "Use only facts in the fields marked as facts and in the photo note. A value marked as the ",
    "owner's decision wins over anything else. If a field is empty, leave that fact out. Never ",
    "guess a designer, model, date or material. If the designer field says the attribution is ",
    "uncertain, write \"attributed to\". Use the era field as it stands. Copy names, model numbers ",
    "and manufacturer names exactly, including Danish letters. From the matching listings take ",
    "only nouns, such as the maker's name, a model number or a type word, at most once in the ",
    "title and once in the description, and never their adjectives or spellings that differ from ",
    "the fields.",
    "\n\n",
    "Write like a careful dealer writing a catalogue entry: plain sentences, concrete nouns, verbs ",
    "like \"is\", \"has\" and \"was made\". Prefer one precise detail to a general claim. Describe ",
    "shape, material and what was done to the piece. Use only adjectives a buyer could check in ",
    "the photos.",
    "\n\n",
    "Title: follow the title formula. Commas only, no words of praise, at most 70 characters.",
    "\n\n",
    "Description: 60 to 120 words, in this order. First, what it is, who made ",
    "it, when and where. Second, one or two sentences on form and materials. Third, what our ",
    "workshop did, naming each restoration step exactly as the fields give it. Fourth, at most one ",
    "sentence on where it can be used.",
    "\n\n",
    "Condition report: at most 45 words. Begin with the restoration in one sentence. Then each ",
    "remaining flaw with its location and approximate size, citing the condition photo number if ",
    "given. Never soften a flaw and never omit one the condition fields list.",
    "\n\n",
    "Search title: at most 60 characters, with the designer or the country, the type, the ",
    "material and the decade; add \" | \" and the shop name only if it still fits. Search ",
    "description: 120 to 155 characters, one or two sentences with what it is, the main ",
    "restoration fact; end with \"Ships from Sweden.\" if it fits.",
    "\n\n",
    "Use commas and full stops. No dashes as punctuation, no exclamation marks or questions, no ",
    "strings of three adjectives. Do not mention AI, the writing process or the shop's values.",
    "\n\n",
    "Follow the style of the examples below but never reuse their sentences. Report ",
    "contradictions you notice as findings, and ask a question only when a fact you need is ",
    "missing or unclear.",
);

/// §9.2, amended by A23 as §9.1.
pub const INSTRUCTION_SV: &str = concat!(
    "Du skriver produkttexter för en liten svensk butik som säljer renoverade skandinaviska ",
    "möbler från mitten av 1900-talet. Du får föremålets fält, en kort anteckning om vad bilderna ",
    "visar, rubriker från andra säljares matchande annonser och ägarens tidigare svar. Skriv ",
    "texterna i `texts`: en rubrik, en beskrivning, en skickrapport, en sökrubrik och en ",
    "sökbeskrivning.",
    "\n\n",
    "Använd bara fakta från fält som är markerade som fakta och från bildanteckningen. Ett värde ",
    "som är ägarens beslut går före allt annat. Är ett fält tomt utelämnar du den uppgiften. ",
    "Gissa aldrig formgivare, modell, årtal eller material. Säger formgivarfältet att ",
    "attributionen är osäker skriver du \"troligen\". Använd tidsfältet som det står. Skriv namn, ",
    "modellnummer och tillverkarnamn exakt, med danska bokstäver, i originalstavning. Ta bara ",
    "substantiv från de matchande annonserna, som tillverkarens namn, ett modellnummer eller ett ",
    "typord, högst en gång i rubriken och en gång i beskrivningen, aldrig deras adjektiv eller ",
    "stavningar som skiljer sig från fälten.",
    "\n\n",
    "Skriv som en noggrann handlare som skriver en katalogtext: enkla meningar, konkreta ",
    "substantiv, verb som \"är\", \"har\" och \"tillverkades\". Välj en exakt detalj framför ett ",
    "allmänt påstående. Beskriv form, material och vad som har gjorts med möbeln. Använd bara ",
    "adjektiv som en köpare kan kontrollera på bilderna. Skriv som en svensk handlare, inte som en ",
    "översättning.",
    "\n\n",
    "Rubrik: följ rubrikformeln. Bara kommatecken, inga berömmande ord, högst 70 tecken, versal ",
    "bara i början och i namn.",
    "\n\n",
    "Beskrivning: 60 till 120 ord, i denna ordning. Först vad det är, vem som ",
    "tillverkade den, när och var. Sedan en eller två meningar om form och material. Sedan vad vår ",
    "verkstad har gjort, med varje renoveringssteg exakt som fälten anger det. Sedan högst en ",
    "mening om var den kan användas.",
    "\n\n",
    "Skickrapport: högst 45 ord. Börja med renoveringen i en mening. Sedan varje kvarvarande ",
    "skada med plats och ungefärlig storlek, med bildnumret om det finns. Förminska aldrig en ",
    "skada och utelämna aldrig en som skickfälten anger.",
    "\n\n",
    "Sökrubrik: högst 60 tecken, med formgivaren eller landet, typen, materialet och decenniet; ",
    "lägg till \" | \" och butikens namn bara om det ryms. Sökbeskrivning: 120 till 155 tecken, en ",
    "eller två meningar med vad det är och den viktigaste renoveringen; avsluta med ",
    "\"Skickas från Sverige.\" om det ryms.",
    "\n\n",
    "Använd kommatecken och punkt. Inga tankstreck, inga utropstecken eller frågor, inga rader med ",
    "tre adjektiv. Nämn aldrig AI, hur texten skrevs eller butikens värderingar.",
    "\n\n",
    "Följ stilen i exemplen nedan men återanvänd aldrig deras meningar. Rapportera motsägelser du ",
    "ser som fynd, och ställ en fråga bara när en uppgift du behöver saknas eller är oklar.",
);

/// The English title formula (evidence §7.2: designer known, designer unknown, attributed).
pub const TITLE_FORMULA_EN: &str = concat!(
    "Title formula (at most 70 characters, title case):\n",
    "- Designer known: {Designer} [{Model}] {Type} in {Material} for {Manufacturer}, {Decade}; add ",
    "\"with {Colour} {Upholstery}\" only within 70 characters; \", Set of {n}\" for sets.\n",
    "- Designer unknown: {Country adj} {Material} {Type} [with {Colour} {Upholstery/seat}], ",
    "{Decade}.\n",
    "- Attributed: {Type} in {Material} attributed to {Designer}, {Decade}.",
);

/// The Swedish title formula (evidence §7.2, Swedish lines; sentence case).
pub const TITLE_FORMULA_SV: &str = concat!(
    "Rubrikformel (högst 70 tecken, versal bara i början och i namn):\n",
    "- Känd formgivare: {Designer} [{modell}] {typ} i {material}, {Tillverkare}, ",
    "{decennium}-tal.\n",
    "- Okänd formgivare: Dansk {material}{typ} [i {färg} {klädsel}], {decennium}-tal.",
);

/// The English dimensions line format.
pub const DIMENSIONS_EN: &str = "Dimensions: W 68 × D 72 × H 76 cm, seat height 42 cm.";
/// The Swedish dimensions line format.
pub const DIMENSIONS_SV: &str = "Mått: B 68 × D 72 × H 76 cm, sitthöjd 42 cm.";

/// A shipped example as constant text ([`Example`] owns its strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShippedExample {
    pub name: &'static str,
    pub fields: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub condition_report: &'static str,
    pub seo_title: &'static str,
    pub seo_description: &'static str,
}

impl ShippedExample {
    pub fn to_example(self) -> Example {
        Example {
            name: self.name.to_owned(),
            fields: self.fields.to_owned(),
            title: self.title.to_owned(),
            description: self.description.to_owned(),
            condition_report: self.condition_report.to_owned(),
            seo_title: self.seo_title.to_owned(),
            seo_description: self.seo_description.to_owned(),
        }
    }
}

/// Evidence §8 A, B, C with the changes of §9.4.
pub const EXAMPLES_EN: [ShippedExample; 3] = [
    ShippedExample {
        name: "A",
        fields: "dining chair; Børge Mogensen; FDB Møbler; J39; 1960s; Denmark; solid beech, \
            papercord; W 49 D 46 H 78, seat 45; light scratches; stable; sanded, oiled, seat \
            rewoven; photo 4 shows scratch.",
        title: "Børge Mogensen J39 Chair in Beech for FDB Møbler, 1960s",
        description: "Model J39 by Børge Mogensen, made in Denmark by FDB Møbler in the 1960s.\n\n\
            The frame is solid beech with a curved top rail, and the seat is woven papercord.\n\n\
            We sanded and oiled the frame and rewove the seat in new natural papercord. It works \
            at a dining table or a desk.\n\n\
            Dimensions: W 49 × D 46 × H 78 cm, seat height 45 cm.",
        condition_report: "Very good, restored. Frame sanded and oiled, seat rewoven with new \
            papercord. A few light scratches remain on the back rail (see photo 4). Stable, with \
            no loose joints.",
        seo_title: "Børge Mogensen J39 Chair in Beech, FDB Møbler 1960s",
        seo_description: "Børge Mogensen J39 dining chair in solid beech, made by FDB Møbler in \
            Denmark in the 1960s. Oiled frame and new papercord seat. Very good condition.",
    },
    ShippedExample {
        name: "B",
        fields: "nesting tables ×3; designer unknown; Denmark; 1960s; teak veneer tops, solid \
            teak legs; light scratches, veneer chip; stable; cleaned, re-oiled; photo 6 shows \
            chip.",
        title: "Danish Teak Nesting Tables, Set of 3, 1960s",
        description: "A set of three nesting tables made in Denmark in the 1960s. There are no \
            maker's marks.\n\n\
            The tops are teak veneer on solid teak legs, and the two smaller tables slide under \
            the largest.\n\n\
            We cleaned and re-oiled all three, keeping the original finish. Use them stacked as \
            one side table or apart as bedside tables.\n\n\
            Dimensions: largest W 54 × D 39 × H 50 cm, smallest W 40 × D 36 × H 42 cm.",
        condition_report: "Good. Cleaned and re-oiled, not refinished. Light scratches on the \
            top of the largest table and a 1 cm veneer chip on the front edge of the middle \
            table (see photo 6). All three stand firm.",
        seo_title: "Danish Teak Nesting Tables, Set of 3, 1960s",
        seo_description: "Set of three Danish teak nesting tables from the 1960s, cleaned and \
            re-oiled with the original finish kept. Good condition. Ships from Sweden.",
    },
    ShippedExample {
        name: "C",
        fields: "sideboard; designer unknown; Denmark; 1960s; teak veneer; W 180 D 45 H 78; \
            water marks, lifted veneer, veneer chips, stiff door; stable; cleaned only; photo 5 \
            shows lifted veneer.",
        title: "Danish Teak Sideboard with Sliding Doors, 1960s",
        description: "A teak sideboard made in Denmark in the 1960s, with two sliding doors \
            over adjustable shelves and four drawers on the right.\n\n\
            It is sold as found. We cleaned it but did not refinish it or repair the veneer, and \
            the price reflects that. It suits a buyer who plans to restore it or is happy with \
            visible wear.\n\n\
            Dimensions: W 180 × D 45 × H 78 cm.",
        condition_report: "Worn, cleaned only. Water marks across the top and a 10 × 3 cm patch \
            of lifted veneer at the back left corner (see photo 5). Two veneer chips along the \
            plinth, the larger about 2 cm. The left door slides stiffly. Structurally sound and \
            stable.",
        seo_title: "Danish Teak Sideboard with Sliding Doors, 1960s",
        seo_description: "Danish teak sideboard from the 1960s with sliding doors, shelves and \
            four drawers. Sold as found and cleaned only. Worn condition. Ships from Sweden.",
    },
];

/// Evidence §8 D with the changes of §9.4.
pub const EXAMPLES_SV: [ShippedExample; 1] = [ShippedExample {
    name: "D",
    fields: "fåtölj; formgivare okänd; Danmark; 1960-tal; massiv teak; raka ben; gräddvit \
        ullbouclé; B 68 D 72 H 76, sitthöjd 42; lätta repor; stadig; slipad, oljad, nyklädd med \
        ny stoppning; bild 5 visar repor.",
    title: "Dansk teakfåtölj i gräddvit bouclé, 1960-tal",
    description: "Fåtölj tillverkad i Danmark på 1960-talet. Formgivare och tillverkare är \
        okända.\n\n\
        Stommen är i massiv teak med svängda armstöd och raka ben. Sits och rygg har ny \
        stoppning och är klädda i gräddvit ullbouclé.\n\n\
        Vi har slipat och oljat stommen. Den passar i ett läshörn eller vid ett skrivbord.\n\n\
        Mått: B 68 × D 72 × H 76 cm, sitthöjd 42 cm.",
    condition_report: "Mycket gott renoverat skick. Stommen slipad och oljad, sits och rygg \
        nyklädda. Lätta repor på vänster armstöd (se bild 5). Stadig, inga glapp i fogarna.",
    seo_title: "Dansk teakfåtölj i ullbouclé, 1960-tal",
    seo_description: "Dansk fåtölj i massiv teak från 1960-talet, slipad och oljad, nyklädd i \
        gräddvit ullbouclé. Mycket gott renoverat skick. Sitthöjd 42 cm.",
}];

/// WC8-R: each "Restoration done" choice with the phrases that state it, English then Swedish
/// (the post-check's rule 11 looks for one of them in the condition report).
pub const RESTORATION_PHRASES: [(&str, &[&str], &[&str]); 14] = [
    ("Cleaned", &["cleaned"], &["rengjord", "rengjorda"]),
    ("Sanded", &["sanded"], &["slipad", "slipade", "slipat"]),
    ("Oiled", &["oiled"], &["oljad", "oljade", "oljat"]),
    ("Waxed", &["waxed"], &["vaxad", "vaxat"]),
    (
        "Lacquered",
        &["lacquered"],
        &["lackad", "lackerad", "lackat"],
    ),
    ("Joints re-glued", &["re-glued"], &["limmad", "limmade"]),
    (
        "Veneer repaired",
        &["veneer repaired", "repaired the veneer"],
        &["lagad fanér", "fanéren lagad"],
    ),
    ("New foam", &["new foam"], &["ny stoppning"]),
    ("New webbing", &["new webbing"], &["nya gjordar"]),
    ("New springs", &["new springs"], &["nya fjädrar"]),
    (
        "New papercord",
        &["new papercord", "rewove"],
        &["nytt pappersgarn", "nyflätad"],
    ),
    (
        "New fabric",
        &["new fabric", "reupholstered"],
        &["nyklädd", "nytt tyg"],
    ),
    ("New leather", &["new leather"], &["nytt läder"]),
    ("Rewired", &["rewired"], &["nya elkablar", "omdragen el"]),
];

/// The phrases of a restoration choice in a language (English for any other); empty for a choice
/// the table does not list.
pub fn restoration_phrases(choice: &str, language: &str) -> &'static [&'static str] {
    RESTORATION_PHRASES
        .iter()
        .find(|(c, _, _)| *c == choice)
        .map_or(&[], |(_, en, sv)| if swedish(language) { sv } else { en })
}

fn swedish(language: &str) -> bool {
    language == "Swedish"
}

/// The shipped instruction of a language (English for any other).
pub fn instruction(language: &str) -> &'static str {
    if swedish(language) {
        INSTRUCTION_SV
    } else {
        INSTRUCTION_EN
    }
}

/// The shipped title formula of a language.
pub fn title_formula(language: &str) -> &'static str {
    if swedish(language) {
        TITLE_FORMULA_SV
    } else {
        TITLE_FORMULA_EN
    }
}

/// The shipped dimensions line of a language.
pub fn dimensions_line(language: &str) -> &'static str {
    if swedish(language) {
        DIMENSIONS_SV
    } else {
        DIMENSIONS_EN
    }
}

/// A23: the sentence a writer switch adds to the instructions, per language: `(english, swedish)`.
pub const GRADE_ON: (&str, &str) = (
    "Condition report: begin with the grade word of the Detailed condition field (Excellent, \
     Very Good, Good or Worn).",
    "Skickrapport: börja med skickordet från fältet Detailed condition (Utmärkt, Mycket gott, \
     Gott eller Slitet skick).",
);
/// A23 (owner 2026-11-19): with the grade switch off, the grade only shapes the tone.
pub const GRADE_OFF: (&str, &str) = (
    "Never write the grade words of the Detailed condition field (Excellent, Very Good, Good, \
     Worn) in any text; let the grade shape the tone only.",
    "Skriv aldrig skickorden från fältet Detailed condition (Utmärkt, Mycket gott, Gott, Slitet \
     skick) i någon text; låt skicket bara styra tonen.",
);
pub const STABILITY_ON: (&str, &str) = (
    "End the condition report with a sentence on stability.",
    "Avsluta skickrapporten med en mening om stabiliteten.",
);
pub const DIMENSIONS_ON: (&str, &str) = (
    "End the description with the dimensions line in the given format, with the seat height \
     for seating.",
    "Avsluta beskrivningen med måttraden i angivet format, med sitthöjd för sittmöbler.",
);

/// The language's half of a switch sentence.
pub fn switch_sentence(sentence: (&'static str, &'static str), language: &str) -> &'static str {
    if swedish(language) {
        sentence.1
    } else {
        sentence.0
    }
}

/// The shipped examples of a language.
pub fn examples(language: &str) -> Vec<Example> {
    if swedish(language) {
        EXAMPLES_SV.iter().map(|e| e.to_example()).collect()
    } else {
        EXAMPLES_EN.iter().map(|e| e.to_example()).collect()
    }
}
