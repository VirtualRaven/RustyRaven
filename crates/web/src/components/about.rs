use dioxus::{prelude::*};

#[component]
pub fn About() -> Element 
{

    rsx! {
        
        document::Title { "SJF Concept - Om butiken" }
        div {
            class: "about",
            h2 {"OM - SJF CONCEPT"}

            p  { "Välkommen till SJF Concept – där kvalitet, precision och kreativitet möts." }

            p { "Vi är ett passionerat företag med fokus på hög standard, noggrannhet och professionellt utförande i allt vi
            gör. Hos oss hittar du ett unikt sortiment av kläder, lasergraverade produkter, 3D-printade föremål,
            dekaler och personligt anpassad design – alltid framtaget med omsorg och öga för detaljer." }

            p { "Vi tror på att varje kund ska få något utöver det vanliga. Oavsett om du letar efter en personlig present,
            vill profilera ditt företag med specialdesignade produkter eller bara uppskattar snygg och genomtänkt
            design – är SJF Concept platsen för dig." }

            p { "Med kombinationen av modern teknik, kreativt tänkande och hantverksskicklighet skapar vi produkter
            som sticker ut och håller över tid. Vårt mål är enkelt: att leverera det lilla extra varje gång." }

            p { "Önskar du något special eller har en fundering så tveka inte att höra av dig till: sjfconcept@hotmail.com"}

            div {
                img {
                    src:  crate::components::footer::FOOTER_LOGO
                },
            }
        }


    }
}