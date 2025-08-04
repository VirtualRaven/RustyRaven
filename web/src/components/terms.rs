
use dioxus::{logger::tracing::info, prelude::*};

#[component]
pub fn TermsAndConditions() -> Element
{
    rsx! {
        div {
            class: "terms",

            h2 { "Användarvillkor för SJF Concept"}

            p { "Senast uppdaterad: 4/8 2025" }


            p {"Genom att handla från vår webshop godkänner du följande villkor. Vi rekommenderar att du läser igenom
            dem innan du genomför ett köp."}

            h3 { "1. Allmänt"}
            p {"Denna webbplats drivs av SJF Concept och tillhandahåller lasergraverade produkter, 3D-printade varor,
            kläder och andra specialanpassade produkter.
            Alla produkter tillverkas enligt beställning och anpassas efter kundens önskemål i den mån det är möjligt."
            }

            h3 {"2. Ansvarsfriskrivning"}
            p {"Vi reserverar oss för eventuella tryckfel, fel i produktbeskrivningar, prisändringar, tillfällig slutförsäljning
            eller tekniska problem på sidan, Vi reserverar oss även för prisändringar orsakat av prisändring från
            leverantörer, samt felaktigheter i priser beroende på felaktig information och förbehåller oss rätten att
            justera priset.
            Vi kan inte hållas ansvariga för skador som uppstår till följd av felaktig användning av produkterna.
            Anpassade produkter kan variera något i utseende, färg och form.
            Vi accepterar inget ansvar för förseningar/fel till följd av omständigheter utanför företagets rådande
            (Force Majeure). Dessa omständigheter kan exempelvis vara arbetskonflikt, eldsvåda, krig,
            myndighetsbeslut, förminskad eller utebliven leverans från leverantör.
            Vidare tas inget ansvar för eventuella förändringar på produkter/produktegenskaper som ändrats av
            respektive leverantör och andra faktorer utanför vår kontroll."}

            h3 {"3. Priser och betalning"}
            p {"Alla priser anges i SEK inklusive moms (25 %) om inget annat anges.
            Betalning sker via våra godkända betalningsmetoder: Swish, kortbetalning, Klarna, faktura
            Beställningen är bindande efter att betalningen har registrerats."}

            h3 {"4. Leveransvillkor"}

            p {"Våra normala leveranstider är 2-7 dagar. OBS! Beställningar lagda på helger skickas tidigast på
            måndagen efter. Vi skickar produkter såfort vi kan, oftast inom 1-2 dagar beroende på produktens
            komplexitet och lagersaldo. Specialbeställningar skickas inom 1-5 dagar beroende på mängd och
            utförande.
            Vid hög belastning (t.ex. högtider) kan leveranstiderna bli längre – vi meddelar dig då via e-post.
            Fraktavgift tillkommer och visas innan du slutför köpet. Leverans sker via PostNord, DHL, Schenker"}

            h3 {"5. Retur & ångerrätt"}
            p{"Vid nyttjande av din ångerrätt:
            Du måste meddela att du ångrar dig. Meddelandet ska skickas till oss sjfconcept@hotmail.com . I ditt
            meddelande ska ditt namn, din adress, e-postadress, ordernummer samt vilka varor som returneringen
            gäller framgå klart och tydligt.
            Du bör omedelbart och senast inom lagstiftad 14 dagar efter ångermeddelandet returnera produkterna till
            oss.
            Du står för returfrakt/leverans och skicket på produkterna vid retur ska vara i det skick du köpte det i,
            produkterna bör därför skickas välpaketerade och i ursprunglig förpackning.
            På återbetalningsbeloppet förbehåller vi oss rätten att dra av en summa motsvarande värdeminskningen
            jämfört med varans ursprungliga värde vid använd eller skadad produkt.
            Anpassade produkter (t.ex. graverade eller specialtillverkade varor) omfattas inte av ångerrätten enligt
            distansavtalslagen (2005:59). Alltså ingen ångerrätt på specialbeställda produkter.
            Om produkten däremot är skadad vid leverans eller om du fått fel vara, kontakta oss såfort du fått hem
            varan eller inom 14 dagar så löser vi det.
            Ej specialanpassade produkter omfattas av 14 dagars öppet köp enligt lag. Returfrakt bekostas av
            kunden."}

            h3 {"6. Reklamation"}
            p {"Om produkten är defekt har du rätt att reklamera. Kontakta oss med bild och beskrivning så återkommer
            vi snarast."}

            h3 {"7. Ändringar av villkor"}
            p{"Vi förbehåller oss rätten att uppdatera dessa villkor när som helst. Ändringar publiceras på hemsidan."}

            h3 {"8. Kontaktuppgifter" }
            p {"Har du frågor är du välkommen att kontakta oss:
            📧 E-post: sjfconcept@hotmail.com
            Svarar på era frågor i snabbast möjliga mån."}

            h2{"🔐 Integritetspolicy (GDPR)"}
            p {"Vi värnar om din personliga integritet och samlar endast in information som är nödvändig för att behandla
            din beställning."}

            h3 {"1. Vilka uppgifter samlar vi in?"}

            div {
                p {"När du handlar från vår sida samlar vi in:"}
                ul {
                    li { "Namn"}
                    li { "Adress"}
                    li { "Telefonnummer"}
                    li { "E-postadress"}
                    li { "Beställningsinformation"}
                    li { "IP-adress (för tekniska/loggändamål)"}
                }

            }

            p { "Vi samlar inte in mer än vad som är nödvändigt för att kunna leverera dina varor."}

            h3 { "2. Hur används uppgifterna?" }

            p {"Uppgifterna används för att:"}
            ul {
                li { "Behandla och leverera din beställning" }
                li { "Hantera betalningar" }
                li { "Skicka orderbekräftelse och ev. uppföljning" }
                li { "Kundtjänstärenden" }
                li { "Uppfylla lagkrav (t.ex. bokföring)" }
            }

            h3 {"3. Tredjepartsdelning"}

            p {"Vi delar endast information med:"}
            ul {
                li {"Betalningslösningar Klarna ,Stripe, Swish "}
                li {"Fraktbolag Postnord, Schenker ,DHL "}
                li {"Bokföringsprogram eller revisorer "}
            }
            p {"Vi säljer aldrig dina uppgifter vidare."}
        }
    }
}