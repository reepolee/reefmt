/**
 * Build-time data for the Services page. The five services are anchored to one
 * substrate per strategy §4: "adopt the Reepolee framework as your development
 * baseline layer — and get the most out of it."
 */

const SERVICES = {
	en: [
		{
			title: "Plan your digital transformation",
			content:
				"No idea where to start? I help your internal teams identify and analyze the business needs and bottlenecks holding you back from realizing your true potential. The end result is a functional specification, a plan of action and a consensus from your stakeholders, which allows them to embrace the change rather than fight it.",
		},
		{
			title: "Develop your app",
			content:
				"How do you make ideas and specs turn into reality? Based on the functional specification and action plan, we scaffold the initial architecture of the application on the Reepolee framework as your baseline. By rapidly shipping incremental changes, we allow for smoother releases and short feedback loops from internal or external stakeholders.",
		},
		{
			title: "Embed into and upscale your team",
			content:
				"Once developed, will we be able to operate the application and improve it over time? Of course! My aim is to have your team fully ready to take over the development whenever they feel ready to do so. Sharing knowledge and upscaling your team rather than vendor-locking you in is the very core belief of mine.",
		},
		{
			title: "Mentor and support your dev talent",
			content:
				"Found an engineer with a high ceiling, but they need a little help reaching it? I have helped teams stuck on a knowledge gap with no bridge to cross it. Regular pairing sessions or changing the main architectural bottlenecks to allow for smoother and easier development and maintenance is one of Reepolee's core competencies.",
		},
		{
			title: "Simplify your operations",
			content:
				"Sick of babysitting servers, app crashes that derail your sales team's efforts and make your customers' lives harder rather than simpler? Reepolee's deployment principles rely on a combination of household name companies and/or \"simple and cost effective\" solutions, that will cost your company a fraction of what you are currently paying in over-provisioned servers and stressed out DevOps teams.",
		},
	],
	sl: [
		{
			title: "Načrtovanje digitalne preobrazbe",
			content:
				"Ne veste, kje začeti? Vašim ekipam pomagam prepoznati in analizirati poslovne potrebe in ozka grla, ki vas ovirajo pri uresničevanju vašega pravega potenciala. Končni rezultat je funkcionalna specifikacija, načrt ukrepov in soglasje zainteresiranih strani, ki jim omogoča, da sprejmejo spremembe in se jim ne upirajo.",
		},
		{
			title: "Razvoj aplikacije",
			content:
				"Kako uresničiti zamisli in specifikacije? Na podlagi funkcionalne specifikacije in akcijskega načrta oblikujemo začetno arhitekturo aplikacije na ogrodju Reepolee kot vaši osnovi. S pogostimi iteracijami hitro pridobimo povratne informacije s strani notranjih ali zunanjih deležnikov.",
		},
		{
			title: "Vključitev v ekipo",
			content:
				"Ali bomo lahko aplikacijo po razvoju nadgrajevali in jo sčasoma izboljševali? Seveda! Moj cilj je, da bo vaša ekipa popolnoma pripravljena prevzeti razvoj, ko se bo za to počutila pripravljeno. Moje temeljno prepričanje je, da si izmenjujemo znanje in širimo vašo ekipo, namesto da bi vas priklenil na dobavitelja.",
		},
		{
			title: "Mentorstvo in podpora razvijalcem",
			content:
				"Ste našli inženirja z visokim potencialom, ki potrebuje malo pomoči? Že prej sem pomagal ekipam, ki so obtičale ob vrzeli v znanju brez mostu čez njo. Redne delavnice ali analiza in spreminjanje glavnih arhitekturnih težav, ki omogočajo nemoten in lažji razvoj in vzdrževanje, so ena od glavnih kompetenc podjetja Reepolee.",
		},
		{
			title: "Poenostavite svoje operacije",
			content:
				"Imate dovolj skrbi za strežnike, nedelujočih aplikacij, ki onemogočajo prodajne ekipe in vašim strankam otežujejo življenje, namesto da bi ga poenostavile? Reepoleejeva načela operacij temeljijo na kombinaciji znanih ponudnikov in/ali preprostih in stroškovno učinkovitih rešitev, ki bodo vaše podjetje stale le delček tistega, kar trenutno plačujete za prevelike strežnike in preobremenjene DevOps ekipe.",
		},
	],
};

export async function load_template_data(): Promise<Record<string, any>> {
	return {
		services: SERVICES,
	};
}
