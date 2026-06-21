/**
 * Build-time data for the Results listing (decision-maker case studies).
 * Full case-study detail pages live under /results/<slug>/.
 *
 * NOTE: outcomes here are deliberately qualitative. Hard metrics
 * (time-to-ship, maintenance hours, onboarding time, infra cost) must be
 * gathered from the client before publishing — never fabricated.
 */

const CASE_STUDIES = {
	en: [
		{
			slug: "back-office",
			image: "/images/responsive/hero-privacy.png",
			context: "Legacy modernization",
			title: "The back-office nobody wanted to touch",
			outcome:
				"A 12-year-old back-office app went from 'don't touch it' to the team's favourite — and onboarding dropped from weeks to days.",
		},
		{
			slug: "upgrade-treadmill",
			image: "/images/responsive/hero-2.png",
			context: "Framework churn",
			title: "Off the upgrade treadmill",
			outcome:
				"Escaping constant churn from underlying-framework releases — to a zero-dependency, owned baseline with a predictable roadmap and no forced migrations.",
		},
		{
			slug: "simplified-operations",
			image: "/images/responsive/hero-contact.png",
			context: "Operations",
			title: "Simplified operations",
			outcome:
				"Over-provisioned servers and fragile deploys gave way to simple, cost-effective ops — real cost savings and fewer incidents.",
		},
	],
	sl: [
		{
			slug: "back-office",
			image: "/images/responsive/hero-privacy.png",
			context: "Modernizacija",
			title: "Zaledni sistem, ki se ga nihče ni želel dotakniti",
			outcome:
				"12 let stara zaledna aplikacija je od 'ne dotikaj se' postala najljubša ekipi — uvajanje se je skrajšalo s tednov na dni.",
		},
		{
			slug: "upgrade-treadmill",
			image: "/images/responsive/hero-2.png",
			context: "Menjava ogrodij",
			title: "Konec nenehnih nadgradenj",
			outcome:
				"Pobeg pred nenehnim spreminjanjem zaradi izdaj osnovnih ogrodij — k lastni osnovi brez odvisnosti, s predvidljivim načrtom in brez vsiljenih migracij.",
		},
		{
			slug: "simplified-operations",
			image: "/images/responsive/hero-contact.png",
			context: "Operacije",
			title: "Poenostavljene operacije",
			outcome:
				"Predimenzionirani strežniki in krhki uvajalni procesi so se umaknili preprostim, stroškovno učinkovitim operacijam — resnični prihranki in manj izpadov.",
		},
	],
};

export async function load_template_data(): Promise<Record<string, any>> {
	return {
		case_studies: CASE_STUDIES,
	};
}
