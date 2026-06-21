/**
 * Build-time data for the Enterprise page.
 *
 * This is an open-source-for-enterprise pitch: Reepolee and its ecosystem tools
 * are free and MIT-licensed; the enterprise plan is how teams that depend on
 * them fund the roadmap and get the project working for their business. The five
 * items render through the shared <service-item> component so the section mirrors
 * the homepage "My Services" block (numbered 01/02/…). Voice matches that copy —
 * first-person, plain-spoken, anti-lock-in — without repeating its sentences.
 */

const OFFERING = {
	en: [
		{
			title: "Sponsor the project",
			content:
				"Reepolee and its ecosystem tools are open source and free to use — and they stay that way. An enterprise sponsorship funds the roadmap you rely on: maintained releases, a healthy project and a baseline that won't go stale underneath your team.",
		},
		{
			title: "Priority support",
			content:
				"Skip the public queue. You get a direct line to the people who build Reepolee — agreed response times, help triaging production issues, and someone who knows the framework inside-out instead of a community thread and a maybe.",
		},
		{
			title: "Development driven by your needs",
			content:
				"We grow the open ecosystem in the direction you're headed. The capabilities your stack needs from Reepolee move up the queue and ship as first-class, maintained parts of the tools — useful to you today and supported for the long run.",
		},
		{
			title: "Custom parts for your stack",
			content:
				"Need something specific to your environment? We build bespoke modules and integrations on top of our ecosystem tools, to Reepolee's standards, then hand them to your team to own — no proprietary strings, no lock-in.",
		},
		{
			title: "Embed and upskill your team",
			content:
				"We work alongside your engineers — pairing, reviewing and running hands-on workshops — until they can plan, build and operate on Reepolee on their own. The whole point is a team that no longer needs us.",
		},
	],
	sl: [
		{
			title: "Sponzorirajte projekt",
			content:
				"Reepolee in njegova ekosistemska orodja so odprtokodna in brezplačna za uporabo — in takšna ostanejo. Podjetniško sponzorstvo financira načrt, na katerega se zanašate: vzdrževane izdaje, zdrav projekt in osnovo, ki se pod vašo ekipo ne postara.",
		},
		{
			title: "Prednostna podpora",
			content:
				"Preskočite javno vrsto. Dobite neposredno povezavo do ljudi, ki gradijo Reepolee — dogovorjene odzivne čase, pomoč pri reševanju produkcijskih težav in nekoga, ki ogrodje pozna do obisti, namesto foruma in morda.",
		},
		{
			title: "Razvoj po vaših potrebah",
			content:
				"Odprti ekosistem razvijamo v smeri, kamor ste namenjeni. Zmožnosti, ki jih vaš sklad potrebuje od Reepolee, se prebijejo višje na seznam in izidejo kot polnopravni, vzdrževani deli orodij — uporabni danes in podprti dolgoročno.",
		},
		{
			title: "Deli po meri za vaš sklad",
			content:
				"Potrebujete nekaj specifičnega za vaše okolje? Zgradimo namenske module in integracije nad našimi ekosistemskimi orodji, po standardih Reepolee, in jih predamo vaši ekipi v last — brez lastniških zank, brez vezave.",
		},
		{
			title: "Vključimo in nadgradimo vašo ekipo",
			content:
				"Delamo ob vaših inženirjih — v paru, s pregledi kode in praktičnimi delavnicami — dokler ne znajo sami načrtovati, graditi in upravljati na Reepolee. Bistvo je ekipa, ki nas ne potrebuje več.",
		},
	],
};

export async function load_template_data(): Promise<Record<string, any>> {
	return {
		offering: OFFERING,
	};
}
