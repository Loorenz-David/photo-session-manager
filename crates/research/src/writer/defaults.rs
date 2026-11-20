//! W2·U1: the shipped writing guide (`master_plan_writer.md` §9; book 08C §7; evidence
//! `publication_writer_research.md` §7, §8): the English and Swedish instructions, the title
//! formulas, the dimensions lines and the style examples. The owner edits the instruction and
//! the examples in Settings → Writer (W4); `None` there means these texts.

use crate::writer::config::Example;

/// §9.1, amended by A23 (2026-11-19): the grade word, the stability sentence, the dimensions
/// line and the short paragraphs moved to the four switches of [`crate::writer::config::WriterSettings`].
/// A29 (owner 2026-11-20): rewritten to the owner's listing rules: singular titles without count,
/// country or years; one-paragraph description opening with provenance; restoration in natural
/// words; a customer condition report without photo numbers or sizes; no shop name in the
/// search title; ask when a field looks wrong.
pub const INSTRUCTION_EN: &str = concat!(
    "You write product texts for a small Swedish shop that sells restored Scandinavian ",
    "mid-century furniture. You receive the item's fields, a short note of what the photos ",
    "show, the titles of matching listings from other sellers, and the owner's earlier ",
    "answers. Write the texts listed in `texts`: a title, a description, a condition report, ",
    "a search title and a search description.",
    "\n\n",
    "Use only facts in the fields marked as facts and in the photo note. A value marked as ",
    "the owner's decision wins over anything else. If a field is empty, leave that fact out. ",
    "Never guess a designer, model, date or material. If the designer field says the ",
    "attribution is uncertain, write \"attributed to\". Use the era field as it stands. Copy ",
    "names, model numbers and manufacturer names exactly, including Danish letters, but write ",
    "names in normal name case even when a field stores them in capitals (\"Ib Kofod-Larsen\", ",
    "not \"IB KOFOD-LARSEN\"). From the matching listings take only nouns, such as the maker's ",
    "name, a model number or a type word, never their adjectives or spellings that differ ",
    "from the fields.",
    "\n\n",
    "Write like a careful dealer talking to a customer: plain sentences, concrete nouns, no ",
    "praise. Write about the materials, which parts are veneer and which are solid wood, the ",
    "upholstery, the restoration and the designer or maker. Do not describe shapes; the ",
    "photos show them. Leave measurements out unless they are the one thing that makes the ",
    "piece distinctive.",
    "\n\n",
    "Title: always in the singular, also for a set (\"Dining Chair\", not \"Dining Chairs\"). The ",
    "designer or the manufacturer when known, the wood, the type and one distinguishing ",
    "detail. Never a number of pieces, \"set\" or \"pair\", a country, years or measurements. At ",
    "most 70 characters.",
    "\n\n",
    "Description: one single paragraph of 40 to 100 words. First a provenance sentence: ",
    "\"Designed by [designer] for [manufacturer] in [country] during the [period].\" Leave out ",
    "the unknown parts; with no designer and no manufacturer write \"Made in [country] during ",
    "the [period].\" Then two or three sentences: the item with its wood, which parts are ",
    "veneer and which are solid when you are sure, for seating the upholstery's type and ",
    "colour (\"black leather\", \"cream woven fabric\"), and the restoration in natural words. ",
    "Never copy the restoration field as it stands, and never write \"completely restored\", ",
    "\"fully restored\", \"full restoration\" or \"the whole piece\"; write for example \"restored ",
    "and refinished\" or \"reupholstered\". For a set, end with \"Sold as a set of [quantity].\" ",
    "Never a sales line for a single piece. At most one sentence on where it can be used, and ",
    "only when it adds something.",
    "\n\n",
    "Condition report: one or two short sentences for the customer, at most 35 words. Start ",
    "with the positives, such as what was restored and that it is ready to use, then the ",
    "marks that remain, in plain words and with where they are. No photo numbers, no sizes ",
    "and no internal notes. Leave out condition fields that say None or N/A.",
    "\n\n",
    "Search title: at most 60 characters, in the singular and unique to this product. ",
    "Manufacturer known: \"[Manufacturer] [wood] [type], [detail]\". Manufacturer unknown: ",
    "\"[Wood] [type] with [detail], [decade]\". Lowercase except names, and never the shop's ",
    "name. Search description: 120 to 155 characters, one or two sentences with what it is ",
    "and the main restoration fact; end with \"Ships from Sweden.\" if it fits.",
    "\n\n",
    "Use commas and full stops. No dashes as punctuation, no exclamation marks or questions, ",
    "no strings of three adjectives. Do not mention AI, the writing process or the shop's ",
    "values.",
    "\n\n",
    "Follow the style of the examples below but never reuse their sentences. When a field ",
    "looks wrong, contradicts another field or the photo note, or is unclear (a misspelt ",
    "maker, a chair with arms typed as a dining chair, an upholstery whose type is not ",
    "given), ask a short question instead of writing around it. Report other contradictions ",
    "as findings.",
);

/// §9.2, amended by A23 and A29 as §9.1.
pub const INSTRUCTION_SV: &str = concat!(
    "Du skriver produkttexter för en liten svensk butik som säljer renoverade skandinaviska ",
    "möbler från mitten av 1900-talet. Du får föremålets fält, en kort anteckning om vad ",
    "bilderna visar, rubriker från andra säljares matchande annonser och ägarens tidigare ",
    "svar. Skriv texterna i `texts`: en rubrik, en beskrivning, en skickrapport, en sökrubrik ",
    "och en sökbeskrivning.",
    "\n\n",
    "Använd bara fakta från fält som är markerade som fakta och från bildanteckningen. Ett ",
    "värde som är ägarens beslut går före allt annat. Är ett fält tomt utelämnar du den ",
    "uppgiften. Gissa aldrig formgivare, modell, årtal eller material. Säger formgivarfältet ",
    "att attributionen är osäker skriver du \"troligen\". Använd tidsfältet som det står. Skriv ",
    "namn, modellnummer och tillverkarnamn exakt, med danska bokstäver, men med vanlig ",
    "versalisering även när fältet har versaler (\"Ib Kofod-Larsen\", inte \"IB KOFOD-LARSEN\"). ",
    "Ta bara substantiv från de matchande annonserna, som tillverkarens namn, ett ",
    "modellnummer eller ett typord, aldrig deras adjektiv eller stavningar som skiljer sig ",
    "från fälten.",
    "\n\n",
    "Skriv som en kunnig handlare som pratar med en kund: enkla meningar, konkreta ",
    "substantiv, inget beröm. Skriv om materialen, vilka delar som är fanér och vilka som är ",
    "massivt trä, klädseln, renoveringen och formgivaren eller tillverkaren. Beskriv inte ",
    "formen; bilderna visar den. Utelämna mått om de inte är det enda som gör möbeln ",
    "särskild. Skriv som en svensk handlare, inte som en översättning.",
    "\n\n",
    "Rubrik: alltid i singular, även för ett set (\"Matstol\", inte \"Matstolar\"). Formgivaren ",
    "eller tillverkaren om den är känd, träslaget, typen och en särskiljande detalj. Aldrig ",
    "antal, \"set\" eller \"par\", land, årtal eller mått. Högst 70 tecken, versal bara i början ",
    "och i namn.",
    "\n\n",
    "Beskrivning: ett enda stycke på 40 till 100 ord. Först en mening om ursprunget: ",
    "\"Formgiven av [formgivare] för [tillverkare] i [land] under [perioden].\" Utelämna det ",
    "som är okänt; utan formgivare och tillverkare skriver du \"Tillverkad i [land] under ",
    "[perioden].\" Sedan två eller tre meningar: föremålet med träslaget, vilka delar som är ",
    "fanér och vilka som är massiva när du är säker, för sittmöbler klädselns typ och färg ",
    "(\"svart läder\", \"gräddvitt vävt tyg\"), och renoveringen med naturliga ord. Skriv aldrig ",
    "av renoveringsfältet som det står, och skriv aldrig \"helt renoverad\" eller ",
    "\"totalrenoverad\"; skriv till exempel \"renoverad och ytbehandlad\" eller \"nyklädd\". För ",
    "ett set avslutar du med \"Säljs som ett set om [antal].\" Aldrig en säljmening för en ",
    "enskild möbel. Högst en mening om var den kan användas, och bara när den tillför något.",
    "\n\n",
    "Skickrapport: en eller två korta meningar till kunden, högst 35 ord. Börja med det ",
    "positiva, som vad som har renoverats och att den är redo att användas, sedan märkena som ",
    "finns kvar, med enkla ord och var de sitter. Inga bildnummer, inga mått och inga interna ",
    "anteckningar. Utelämna skickfält som säger None eller N/A.",
    "\n\n",
    "Sökrubrik: högst 60 tecken, i singular och unik för den här produkten. Känd tillverkare: ",
    "\"[Tillverkare] [träslag] [typ], [detalj]\". Okänd tillverkare: \"[Typ] i [träslag] med ",
    "[detalj], [decennium]-tal\". Gemener utom i namn, och aldrig butikens namn. ",
    "Sökbeskrivning: 120 till 155 tecken, en eller två meningar med vad det är och den ",
    "viktigaste renoveringen; avsluta med \"Skickas från Sverige.\" om det ryms.",
    "\n\n",
    "Använd kommatecken och punkt. Inga tankstreck, inga utropstecken eller frågor, inga ",
    "rader med tre adjektiv. Nämn aldrig AI, hur texten skrevs eller butikens värderingar.",
    "\n\n",
    "Följ stilen i exemplen nedan men återanvänd aldrig deras meningar. När ett fält ser fel ",
    "ut, motsäger ett annat fält eller bildanteckningen, eller är oklart (ett felstavat ",
    "tillverkarnamn, en stol med armstöd som är angiven som matstol, en klädsel utan typ), ",
    "ställ en kort fråga i stället för att skriva runt det. Rapportera andra motsägelser som ",
    "fynd.",
);

/// The English title formula (evidence §7.2; A29: singular, no count, country or decade).
pub const TITLE_FORMULA_EN: &str = concat!(
    "Title formula (at most 70 characters, title case, always in the singular):",
    "\n",
    "- Designer known: {Designer} [{Model}] {Material} {Type} [for {Manufacturer}] [with ",
    "{detail}].",
    "\n",
    "- Designer unknown, manufacturer known: {Manufacturer} {Material} {Type} [with ",
    "{detail}].",
    "\n",
    "- Neither known: {Material} {Type} with {Colour} {Upholstery} or another {detail}.",
    "\n",
    "- Attributed: {Material} {Type} attributed to {Designer}.",
    "\n",
    "Never a number of pieces, \"Set\" or \"Pair\", a country, a decade or measurements.",
);

/// The Swedish title formula (A29 as the English one; sentence case).
pub const TITLE_FORMULA_SV: &str = concat!(
    "Rubrikformel (högst 70 tecken, versal bara i början och i namn, alltid i singular):",
    "\n",
    "- Känd formgivare: {Designer} [{modell}] {typ} i {material} [för {Tillverkare}] [med ",
    "{detalj}].",
    "\n",
    "- Okänd formgivare, känd tillverkare: {Tillverkare} {typ} i {material} [med {detalj}].",
    "\n",
    "- Ingen känd: {Typ} i {material} med {färg} {klädsel} eller en annan {detalj}.",
    "\n",
    "Aldrig antal, \"set\" eller \"par\", land, decennium eller mått.",
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

/// Evidence §8 A, B, C, rewritten by A29: one paragraph, no grade words, a customer condition
/// report without photo numbers or sizes, singular titles.
pub const EXAMPLES_EN: [ShippedExample; 3] = [
    ShippedExample {
        name: "A",
        fields: concat!(
            "dining chair; Børge Mogensen; FDB Møbler; J39; 1960s; Denmark; solid beech, ",
            "papercord; quantity 4; W 49 D 46 H 78, seat 45; light scratches; stable; sanded, ",
            "oiled, seat rewoven; photo 4 shows scratch.",
        ),
        title: "Børge Mogensen J39 Beech Dining Chair for FDB Møbler",
        description: concat!(
            "Designed by Børge Mogensen for FDB Møbler in Denmark during the 1960s. A J39 ",
            "dining chair in solid beech throughout, with a seat woven in new natural ",
            "papercord. Our workshop sanded and oiled the frames and rewove the seats, so the ",
            "chairs are ready for daily use again. Sold as a set of 4.",
        ),
        condition_report: concat!(
            "Sanded, oiled and rewoven, ready to use. A few light scratches remain on the ",
            "back rail.",
        ),
        seo_title: "FDB Møbler beech dining chair, J39 by Børge Mogensen",
        seo_description: concat!(
            "Børge Mogensen J39 dining chair in solid beech, made by FDB Møbler in Denmark in ",
            "the 1960s. Oiled frame and new papercord seat. Ships from Sweden.",
        ),
    },
    ShippedExample {
        name: "B",
        fields: concat!(
            "nest of tables, three tables; designer unknown; Denmark; 1960s; teak veneer ",
            "tops, solid teak legs; light scratches, veneer chip; stable; cleaned, re-oiled; ",
            "photo 6 shows chip.",
        ),
        title: "Teak Nest of Tables with Veneer Tops",
        description: concat!(
            "Made in Denmark during the 1960s. A teak nest of three tables with veneer tops ",
            "on solid teak legs, the two smaller tables sliding under the largest. We cleaned ",
            "and re-oiled all three and kept the original finish, so the teak keeps the warm ",
            "tone of its age. Stacked they take little room, and apart they make handy side ",
            "tables.",
        ),
        condition_report: concat!(
            "Cleaned and re-oiled, ready to use. Light scratches on the largest top and a ",
            "small veneer chip on the front edge of the middle table.",
        ),
        seo_title: "Teak nest of tables with veneer tops, 1960s",
        seo_description: concat!(
            "Danish teak nest of three tables from the 1960s, cleaned and re-oiled with the ",
            "original finish kept. Veneer tops on solid teak legs. Ships from Sweden.",
        ),
    },
    ShippedExample {
        name: "C",
        fields: concat!(
            "sideboard; designer unknown; Denmark; 1960s; teak veneer; W 180 D 45 H 78; water ",
            "marks, lifted veneer, veneer chips, stiff door; stable; cleaned only; photo 5 ",
            "shows lifted veneer.",
        ),
        title: "Teak Sideboard with Sliding Doors and Drawers",
        description: concat!(
            "Made in Denmark during the 1960s. A teak sideboard with a veneered body, two ",
            "sliding doors over adjustable shelves and four drawers on the right. It is sold ",
            "as found: we cleaned it but did not refinish it or repair the veneer, and the ",
            "price reflects that. It suits a buyer who plans to restore it or is happy with ",
            "visible wear.",
        ),
        condition_report: concat!(
            "Cleaned and sound, sold as found. Water marks on the top, lifted veneer at the ",
            "back left corner, two veneer chips on the plinth and a stiff left door.",
        ),
        seo_title: "Teak sideboard with sliding doors and drawers, 1960s",
        seo_description: concat!(
            "Danish teak sideboard from the 1960s with sliding doors, shelves and four ",
            "drawers. Sold as found and cleaned only, ready to restore. Ships from Sweden.",
        ),
    },
];

/// Evidence §8 D, rewritten by A29 as A, B, C.
pub const EXAMPLES_SV: [ShippedExample; 1] = [ShippedExample {
    name: "D",
    fields: concat!(
        "fåtölj; formgivare okänd; Danmark; 1960-tal; massiv teak; gräddvit ullbouclé; B ",
        "68 D 72 H 76, sitthöjd 42; lätta repor; stadig; slipad, oljad, nyklädd med ny ",
        "stoppning; bild 5 visar repor.",
    ),
    title: "Fåtölj i teak med gräddvit ullbouclé",
    description: concat!(
        "Tillverkad i Danmark under 1960-talet. En fåtölj med stomme, armstöd och ben i ",
        "massiv teak, och sits och rygg klädda i gräddvit ullbouclé. Vi har slipat och ",
        "oljat stommen, och sits och rygg har fått ny stoppning och nytt tyg. Den passar ",
        "i ett läshörn eller vid ett skrivbord.",
    ),
    condition_report: "Slipad, oljad och nyklädd, redo att användas. Lätta repor på vänster armstöd.",
    seo_title: "Fåtölj i massiv teak med ullbouclé, 1960-tal",
    seo_description: concat!(
        "Dansk fåtölj i massiv teak från 1960-talet, slipad och oljad och nyklädd i ",
        "gräddvit ullbouclé med ny stoppning. Skickas från Sverige.",
    ),
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
