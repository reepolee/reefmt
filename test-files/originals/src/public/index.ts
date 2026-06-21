/**
 * Build-time data loader for the homepage.
 *
 * Ports the content that the SvelteKit site kept in `home-page-data.js`
 * (services, testimonials) plus the per-language section copy that used to
 * live inline in each `$sections/*.svelte` component. All copy is keyed by
 * language; the template picks `props.home[props.lang]` etc.
 */

const MY_SERVICES = {
	en: [
		{
			title: "Plan your digital transformation",
			content:
				"No idea where to start? I help your internal teams identify and analyze the business needs and bottlenecks holding you back from realizing your true potential. The end result is a functional specification, a plan of action and a consensus from your stakeholders, which allows them to embrace the change rather than fight it.",
		},
		{
			title: "Develop your app",
			content:
				"How do you make ideas and specs turn into reality? Based on the functional specification and action plan, we scaffold the initial architecture of the application. By rapidly shipping incremental changes to the application, we allow for smoother releases and short feedback loops from internal or external stakeholders.",
		},
		{
			title: "Embed into and upscale your team",
			content:
				"Once developed, will we be able to operate the application and improve it over time? Of course! My aim is to have your team fully ready to take over the development of the application whenever they feel ready to do so. Sharing knowledge and upscaling your team rather than vendor-locking you in is the very core belief of mine.",
		},
		{
			title: "Mentor and support your dev talent",
			content:
				"Found an engineer with a high ceiling, but they need a little help reaching it? I have helped teams in this situation before, stuck on a knowledge gap with no bridge to cross it. Regular pairing sessions or changing the main architectural bottlenecks to allow for smoother and easier development and maintenance is one of Reepolee's core competencies.",
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
				"Kako uresničiti zamisli in specifikacije? Na podlagi funkcionalne specifikacije in akcijskega načrta oblikujemo začetno arhitekturo aplikacije. S pogostimi iteracijami hitro pridobimo povratne informacije s strani notranjih ali zunanjih deležnikov.",
		},
		{
			title: "Vključitev v ekipo",
			content:
				"Ali bomo lahko aplikacijo po razvoju nadgrajevali in jo sčasoma izboljševali? Seveda! Moj cilj je, da bo vaša ekipa popolnoma pripravljena prevzeti razvoj aplikacije, ko se bo za to počutila pripravljeno. Moje temeljno prepričanje je, da si izmenjujemo znanje in širimo vašo ekipo, namesto da bi vas priklenil na dobavitelja.",
		},
		{
			title: "Mentorstvo in podpora razvijalcem",
			content:
				"Ste našli inženirja z visokim potencialom, ki potrebuje malo pomoči, da ga doseže? Že prej sem pomagal ekipam v takšnem položaju, ko so obtičale v času in tehnologiji. Redne delavnice ali analiza in spreminjanje glavnih arhitekturnih težav, ki omogočajo nemoten in lažji razvoj in vzdrževanje, so ena od glavnih kompetenc podjetja Reepolee.",
		},
		{
			title: "Poenostavite svoje operacije",
			content:
				"Imate dovolj skrbi za strežnike, nedelujočih aplikacij, ki onemogočajo prodajne ekipe in vašim strankam otežujejo življenje, namesto da bi ga poenostavile? Reepoleejeva načela operacij temeljijo na kombinaciji znanih ponudnikov in/ali preprostih in stroškovno učinkovitih rešitev, ki bodo vaše podjetje stale le delček tistega, kar trenutno plačujete za prevelike strežnike in preobremenjene DevOps ekipe.",
		},
	],
};

const TESTIMONIALS = {
	en: [
		{
			person: "Goran Mrvoš",
			title: "CEO & Founder @ Infosit",
			logo: "/assets/testimonials/infosit/logo-blue.svg",
			photo: "/images/responsive/infosit-goran-mrvos.jpg",
			linkedin_url: "https://www.linkedin.com/in/goranmrvos/",
			company_url: "https://www.infosit.com/",
			company_linkedin_url: "https://www.linkedin.com/company/infosit",
			company: "infosit",
			slug: "infosit",
			content:
				"We genuinely enjoy working with Aleš. Our partnership on projects allows us to enhance our knowledge and deepen our understanding of the modern SvelteKit framework, which ultimately helps us improve our technical skills. Moreover, our joint efforts contribute to the overall success of the projects.",
		},
		{
			person: "Gregor Rupnik",
			title: "CEO & Partner @ Creatim",
			logo: "/assets/testimonials/creatim/creatim-logo.svg",
			photo: "/images/responsive/creatim-photo.jpg",
			linkedin_url: "https://www.linkedin.com/in/gregor-rupnik/",
			company_url: "https://www.creatim.com/",
			company: "creatim",
			slug: "creatim",
			content:
				"Working with Aleš was extremely pleasant and productive. His inexhaustible energy, innovation and passion for technology have always been an inspiration to the entire team. Aleš is extremely proactive and quickly finds creative solutions even for more demanding IT challenges. I have no hesitation in recommending him to anyone looking for a reliable and highly motivated IT professional.",
		},
		{
			person: "Natalija Premužič",
			title: "CEO @ Super Glavce, Lead of FIRST® (For Inspiration and Recognition of Science and Technology) LEGO® League Programme",
			logo: "/assets/testimonials/superglavce/superglavce-logo.svg",
			photo: "/images/responsive/superglavce-photo.jpg",
			linkedin_url: "https://www.linkedin.com/in/natalija-premu%C5%BEi%C4%8D-a17b7126",
			company_url: "https://www.superglavce.org",
			company: "superglavce",
			slug: "superglavce",
			content:
				"Aleš is an exceptional expert who understands that applications, programs and IT equipment should be adapted to people and not the other way around. He has a wealth of knowledge and experience, he is an excellent advisor and listener. For every challenge, problem or desire that arises, he looks for and finds a solution. Priceless! We have been working together since 2011. Thank you for helping. Aleš is an indispensable member of the program and projects that we run together in Slovenia, Croatia, Serbia and Montenegro.",
		},
		{
			person: "Dr. Janez Križan",
			title: "Founder @ AMI",
			logo: "/assets/testimonials/ami/ami-logo.svg",
			photo: "/images/responsive/ami-photo.jpg",
			linkedin_url: "https://www.linkedin.com/in/janez-krizan-322bb129/",
			company_url: "https://www.amicrystal.com/past-projects",
			company: "ami",
			slug: "ami",
			content:
				"Our thirty-year long cooperation has been very productive in various areas, from the development of specific software applications to work with databases and process control. I hope that despite the years, we will continue to work together for some time.",
		},
		{
			person: "A New Friend",
			title: "",
			logo: "",
			photo: "/images/responsive/next-photo.jpg",
			linkedin_url: "",
			company_url: "",
			company: "next",
			slug: "virtual-next-customer",
			content: "You could be the next one writing a successful story.",
		},
	],
	sl: [
		{
			person: "Goran Mrvoš",
			title: "CEO & Founder @ Infosit",
			logo: "/assets/testimonials/infosit/logo-blue.svg",
			photo: "/images/responsive/infosit-goran-mrvos.jpg",
			linkedin_url: "https://www.linkedin.com/in/goranmrvos/",
			company_url: "https://www.infosit.com/",
			company_linkedin_url: "https://www.linkedin.com/company/infosit",
			company: "infosit",
			slug: "infosit",
			content:
				"Delo z Alešem je resnično zadovoljstvo. Sodelovanje nam omogoča, da širimo svoje znanje in poglabljamo razumevanje sodobnega ogrodja SvelteKit, kar nam na koncu pomaga izpiliti tehnične veščine. Poleg tega naša skupna prizadevanja prispevajo k splošnemu uspehu projekta.",
		},
		{
			person: "Gregor Rupnik",
			title: "Direktor & partner @ Creatim",
			logo: "/assets/testimonials/creatim/creatim-logo.svg",
			photo: "/images/responsive/creatim-photo.jpg",
			linkedin_url: "https://www.linkedin.com/in/gregor-rupnik/",
			company_url: "https://www.creatim.com/",
			company: "creatim",
			slug: "creatim",
			content:
				"Sodelovanje z Alešem je bilo izjemno prijetno in produktivno. Njegova neusahljiva energija, inovativnost in strast do tehnologije so bili vedno navdih za celotno ekipo. Aleš je izjemno proaktiven in hitro najde ustvarjalne rešitve tudi za zahtevnejše IT izzive. Brez zadržkov ga priporočam vsakomur, ki išče zanesljivega in zelo motiviranega IT strokovnjaka.",
		},
		{
			person: "Natalija Premužič",
			title: "Direktorica Zavoda Super Glavce in vodja programa FIRST® (For Inspiration and Recognition of Science and Technology) LEGO® Liga",
			logo: "/assets/testimonials/superglavce/superglavce-logo.svg",
			photo: "/images/responsive/superglavce-photo.jpg",
			linkedin_url: "https://www.linkedin.com/in/natalija-premu%C5%BEi%C4%8D-a17b7126",
			company_url: "https://www.superglavce.org",
			company: "superglavce",
			slug: "superglavce",
			content:
				"Aleš je izjemen strokovnjak, ki razume, da so aplikacije, programi in IT oprema namenjeni ljudem in ne obratno. Ima ogromno znanja, izkušenj, je odličen svetovalec in poslušalec. Za vsak izziv, težavo ali željo, ki se pojavi išče in najde rešitev. Neprecenljivo! Sodelujemo od leta 2011. Hvaležna, ker pomaga. Aleš je nepogrešljiv član za program in projekte, ki jih skupaj vodimo v Sloveniji, na Hrvaškem, v Srbiji in Črni gori.",
		},
		{
			person: "Dr. Janez Križan",
			title: "Ustanovitelj AMI",
			logo: "/assets/testimonials/ami/ami-logo.svg",
			photo: "/images/responsive/ami-photo.jpg",
			linkedin_url: "https://www.linkedin.com/in/janez-krizan-322bb129/",
			company_url: "https://www.amicrystal.com/past-projects",
			company: "ami",
			slug: "ami",
			content:
				"Najino trideset letno sodelovanje je bilo zelo dobro na različnih področjih od razvoja specifičnih programskih aplikacij do dela s podatkovnimi bazami in procesnim krmiljenjem. Upam, da bova kljub letom še nekaj časa sodelovala.",
		},
		{
			person: "Nov prijatelj",
			title: "",
			logo: "",
			photo: "/images/responsive/next-photo.jpg",
			linkedin_url: "",
			company_url: "",
			company: "next",
			slug: "virtual-next-customer",
			content: "Boste vi del naslednje uspešne zgodbe?",
		},
	],
};

// Homepage Results strip — three anchor case studies (summaries; full pages live under /results/*).
const RESULTS_TEASER = {
	en: [
		{
			context: "Legacy modernization",
			outcome: "The back-office nobody wanted to touch — now the team's favourite app.",
			slug: "back-office",
			image: "/images/responsive/hero-privacy.png",
		},
		{
			context: "Framework churn",
			outcome: "Off the upgrade treadmill — a stable, owned baseline with no forced migrations.",
			slug: "upgrade-treadmill",
			image: "/images/responsive/hero-2.png",
		},
		{
			context: "Operations",
			outcome: "Simplified operations — fewer incidents, at a fraction of the infra cost.",
			slug: "simplified-operations",
			image: "/images/responsive/hero-contact.png",
		},
	],
	sl: [
		{
			context: "Modernizacija",
			outcome: "Zaledni sistem, ki se ga nihče ni želel dotakniti — zdaj najljubša aplikacija ekipe.",
			slug: "back-office",
			image: "/images/responsive/hero-privacy.png",
		},
		{
			context: "Menjava ogrodij",
			outcome: "Konec nenehnih nadgradenj — stabilna, lastna osnova brez vsiljenih migracij.",
			slug: "upgrade-treadmill",
			image: "/images/responsive/hero-2.png",
		},
		{
			context: "Operacije",
			outcome: "Poenostavljene operacije — manj izpadov in delček prejšnjih stroškov infrastrukture.",
			slug: "simplified-operations",
			image: "/images/responsive/hero-contact.png",
		},
	],
};

// Homepage Tools teaser — the two peer products (Reeweb static, the full-stack framework).
const PRODUCTS = {
	en: [
		{
			name: "Reeweb",
			badge: "Static",
			tagline: "Static sites with .ree templates, i18n, markdown, RSS and sitemaps. The easy first rung.",
			href: "/reeweb",
			cta: "Start static",
			caption: "screenshot — Reeweb site",
		},
		{
			name: "Reepolee framework",
			badge: "Full-stack",
			tagline:
				"Your development baseline: SSR, CRUD & schema generators, auth and modules — fully owned, zero lock-in.",
			href: "/reepolee",
			cta: "Go full-stack",
			caption: "screenshot — framework app",
		},
	],
	sl: [
		{
			name: "Reeweb",
			badge: "Statično",
			tagline: "Statične strani z .ree predlogami, i18n, markdownom, RSS in sitemapi. Enostavna prva stopnička.",
			href: "/reeweb",
			cta: "Začni statično",
			caption: "posnetek — stran Reeweb",
		},
		{
			name: "Ogrodje Reepolee",
			badge: "Full-stack",
			tagline:
				"Vaša razvojna osnova: SSR, CRUD in generatorji shem, avtentikacija in moduli — v vaši lasti, brez vezave.",
			href: "/reepolee",
			cta: "Polni sklad",
			caption: "posnetek — aplikacija na ogrodju",
		},
	],
};

export async function load_template_data(): Promise<Record<string, any>> {
	const years_of_experience = new Date().getFullYear() - 1986;
	const tailwind_version = "4.3.0";

	const home = {
		en: {
			hero_h1: "Senior software partners — **and the open framework we build with**.",
			hero_results_cta: "See results",
			hero_contact_cta: "Let's talk",
			hero_tools_cta: "Explore the tools",
			results_h2: "**Results**, not promises",
			results_intro:
				"Real outcomes for teams in the public and private sector — the proof behind the philosophy.",
			results_all: "See all results",
			tools_h2: "We **practice what we preach**",
			tools_intro:
				"We ship the simple, zero-dependency, no-lock-in approach as open source. Two products, one engine — start with Reeweb, grow into the framework.",
			tools_engine_line: "Both run on Ree, our own templating engine.",
			tools_all: "Explore the tools",
			tools_more_h: "The rest of the toolbox",
			tools_more_intro:
				"Small, sharp tools that support the two products and keep the workflow consistent — each with its own documentation.",
			tools_more_cta: "Read the docs",
			tools_more: [
				{
					name: "Reefmt",
					badge: "CLI",
					desc: "Fast, dependency-free formatter for .ree templates. A single Rust binary.",
					href: "/reefmt/docs",
				},
				{
					name: "SQLfmt",
					badge: "CLI",
					desc: "Deterministic SQL formatter. Rust, fast, zero configuration.",
					href: "/sqlfmt/docs",
				},
				{
					name: "Reemerge",
					badge: "CLI",
					desc: "Interactive cherry-pick for preparing clean, hand-picked pull requests. A single Rust binary.",
					href: "/reemerge/docs",
				},
				{
					name: "Ree Templates for VSCode",
					badge: "Editor",
					desc: "Syntax highlighting, icons and format-on-save for .ree files.",
					href: "/ree-templates-vscode/docs",
				},
			],
			short_h3:
				"Whether you need help with **product discovery, breaking down** and **planning your software development** or **choosing the right approach** to better utilize your current or growing development team, **Reepolee has your back**.",
			short_text:
				"Our hands-on consulting services, paired with initial project scaffolding have helped many clients reach their full potential, ship faster and increase their customer satisfaction, while vastly simplifying their tech stack, operations and development cost.",
			ales_h1: "**Need a partner**, not just another agency?",
			ales_lg: "**Hi, I'm Aleš and I run Reepolee.**",
			ales_2: "Since 1986, I have helped our clients - entities in the public and private sector - identify the bottlenecks in their software-based products and processes. I believe in wholesome but simple battle-tested solutions, with intense focus on the UX, DX and CX.",
			ales_3: "Reepolee however is not a classic agency. Rather than locking-in our clients, I help them to build up their internal development skills and knowledge to a degree where they can fully take over, thus maintaining complete control of their own development lifecycle.",
			ales_alt: "Headshot of Aleš",
			lets_talk: "Let's talk",
			success_xl: "**Key to Success**",
			success_1:
				"In the fast evolving World, the companies that win are able to focus on the imperceptible details, iterate rapidly, keep to a realistic schedule and deliver world class products and experiences to the client.",
			success_2:
				"My strength lies in being able to take business requirements and translate them into a complete functional specification that all stakeholders can understand, but more importantly development and design teams can turn into well-tested and beautifully written code.",
			years_experience: "years of experience in leadership and software engineering",
			completed_projects: "completed projects",
			clients: "clients",
			services_h1: "My<br>**Services**",
			services_1:
				"Digital transformation touches every aspect of your business. Over the course of the past " +
				years_of_experience +
				" years, I have developed a process that allows companies to plan their digital transformation along with all of the stakeholders, develop the plan of action and build the necessary digital tooling to support it.",
			stack_h1: "**Our own stack.**<br>Modern, but simple",
			stack_1:
				"The stack I use is simple to learn and cost-effective to maintain, highly customizable to the needs of your business and clients, while never getting in the way of exceptional user experience and high level of developer satisfaction and retention.",
			stack_framework:
				"At the centre is the Reepolee framework and Ree, our own zero-dependency templating engine. A familiar, minimal syntax and a tiny dependency tree mean fewer surprises, faster onboarding and a baseline your team fully owns — no vendor lock-in.",
			stack_bun_1:
				"We build on Bun today because we like where it's heading — it's fast, batteries-included and a genuine selling point right now. Rather than tying the framework's identity to any single runtime, we keep the stack adaptable — as the ecosystem evolves, we evolve with it, not against it.",
			stack_tw_1:
				"Tailwind CSS is THE solution to long-term CSS maintenance. I've been a follower since the first alpha and this site is actually using",
			stack_tw_2:
				". It pairs perfectly with Svelte components and allows for code proximity and mental visualization even before rendering.",
			stack_cf_1:
				"When clients ask for a provider to run their code in the most secure manner while being extremely fast and keeping housekeeping at the minimum, I most often recommend Cloudflare. If an application qualifies to be run on their infrastructure, they are my number one choice. With a generous free tier my clients can easily try their service and gain confidence while we're developing the app so by the time they go live, they are already familiar with it and can upgrade to paid tiers when needed.",
			stack_cf_2: "Btw, they serve this page from",
			stack_cf_3: "probably the closest to you.",
			stack_other: "Here are some other tools we've used before and love them as well, in no particular order",
			stack_disclaimer:
				"Disclaimer: I am not affiliated, associated, authorized, endorsed by, or in any way officially connected with the forementioned projects or their holding companies. I just really like their products or services and recommend them openheartedly.",
			testimonials_h1: "How clients<br>**see me**",
		},
		sl: {
			hero_h1: "Izkušeni programski partnerji — **in odprto ogrodje, s katerim gradimo**.",
			hero_results_cta: "Poglej rezultate",
			hero_contact_cta: "Kontakt",
			hero_tools_cta: "Razišči orodja",
			results_h2: "**Rezultati**, ne obljube",
			results_intro: "Resnični dosežki za ekipe v javnem in zasebnem sektorju — dokaz za filozofijo.",
			results_all: "Vsi rezultati",
			tools_h2: "**Sami uporabljamo, kar pridigamo**",
			tools_intro:
				"Preprost pristop brez odvisnosti in brez vezave objavljamo kot odprto kodo. Dva izdelka, en pogon — začnite z Reeweb in zrastite v ogrodje.",
			tools_engine_line: "Oba poganja Ree, naš lastni pogon za predloge.",
			tools_all: "Razišči orodja",
			tools_more_h: "Preostala orodja",
			tools_more_intro:
				"Majhna, ostra orodja, ki podpirajo oba izdelka in ohranjajo dosleden potek dela — vsako s svojo dokumentacijo.",
			tools_more_cta: "Dokumentacija",
			tools_more: [
				{
					name: "Reefmt",
					badge: "CLI",
					desc: "Hiter formater za .ree predloge brez odvisnosti. Ena sama izvedljiva datoteka v Rustu.",
					href: "/reefmt/docs",
				},
				{
					name: "SQLfmt",
					badge: "CLI",
					desc: "Determinističen formater za SQL. Rust, hiter, brez nastavitev.",
					href: "/sqlfmt/docs",
				},
				{
					name: "Reemerge",
					badge: "CLI",
					desc: "Interaktivni cherry-pick za pripravo čistih, ročno izbranih pull requestov. Ena sama izvedljiva datoteka v Rustu.",
					href: "/reemerge/docs",
				},
				{
					name: "Ree Templates za VSCode",
					badge: "Urejevalnik",
					desc: "Barvanje sintakse, ikone in oblikovanje ob shranjevanju za .ree datoteke.",
					href: "/ree-templates-vscode/docs",
				},
			],
			short_h3:
				"Ne glede na to, ali potrebujete pomoč pri **definiranju funkcionalnosti**, **načrtovanju razvoja** programske opreme ali **izbiri pravega pristopa** za boljši **izkoristek** vaše trenutne ali rastoče razvojne **ekipe**, vam **Reepolee vedno stoji ob strani**.",
			short_text:
				"Naše svetovalne storitve so v kombinaciji z začetnim projektnim ogrodjem že številnim strankam pomagale, da so dosegle svoj polni potencial, hitreje dokončale projekte in povečale zadovoljstvo svojih strank, hkrati pa poenostavile svoj tehnološki sklad, operacije in stroške razvoja.",
			ales_h1: "**Potrebujete partnerja** namesto še ene agencije?",
			ales_lg: "**Zdravo, jaz sem Aleš in vodim Reepolee**",
			ales_2: "Od leta 1986 pomagam strankam v javnem in zasebnem sektorju pri odkrivanju ozkih grl na projektih, ki temeljijo na programski opremi in procesih. Verjamem v enostavne in preverjene rešitve, pri čemer se intenzivno osredotočam na uporabniške in programerske izkušnje.",
			ales_3: "Vendar pa Reepolee ni klasična agencija. Namesto da bi naše stranke zaklenil na določeno tehnologijo, jim pomagam, da razvijejo lastne razvojne ekipe in znanja do te mere, da jih lahko v celoti prevzamejo in tako ohranijo popoln nadzor nad razvojnim ciklom.",
			ales_alt: "Alešev portret",
			lets_talk: "Kontakt",
			success_xl: "**Ključ do uspeha**",
			success_1:
				"V hitro razvijajočem se svetu so se zmagovalna podjetja sposobna osredotočiti na neopazne podrobnosti, hitro spreminjati procese, se držati realnih časovnic ter strankam zagotoviti vrhunske izdelke in izkušnje.",
			success_2:
				"Moja prednost je v tem, da lahko povzamem poslovne zahteve in jih prevedem v popolno funkcionalno specifikacijo, ki jo razumejo vse zainteresirane strani. Še pomembneje pa je, da jo lahko oblikovalske in razvojne ekipe spremenijo v enostavno in preizkušeno kodo.",
			years_experience: "let izkušenj na področju vodenja in inženiringa programske opreme",
			completed_projects: "dokončanih projektov",
			clients: "strank",
			services_h1: "Moje<br>**Storitve**",
			services_1:
				"Digitalna preobrazba se dotika vseh vidikov poslovanja. V preteklih " +
				years_of_experience +
				" letih sem razvil proces, ki podjetjem omogoča, da skupaj z vsemi deležniki načrtujejo digitalno preobrazbo, razvijejo akcijski načrt in zgradijo potrebna digitalna orodja za podporo.",
			stack_h1: "**Naš lastni sklad.**<br>Moderno, a enostavno",
			stack_1:
				"Sklad, ki ga uporabljam, je preprost za učenje in stroškovno učinkovit za vzdrževanje, zelo prilagodljiv potrebam vašega podjetja in strank, hkrati pa nikoli ne ovira izjemne uporabniške izkušnje ter visoke ravni zadovoljstva razvijalcev.",
			stack_framework:
				"V središču sta ogrodje Reepolee in Ree, naš lastni pogon za predloge brez odvisnosti. Znan, minimalen sintaks in majhno drevo odvisnosti pomenijo manj presenečenj, hitrejše uvajanje in osnovo, ki je v celoti v lasti vaše ekipe — brez vezave na dobavitelja.",
			stack_bun_1:
				"Danes gradimo na Bun, ker nam je všeč njegova smer — je hiter, opremljen z vsem potrebnim in trenutno resnična prednost. Namesto da bi identiteto ogrodja vezali na en sam runtime, ohranjamo sklad prilagodljiv — z razvojem ekosistema se razvijamo tudi mi.",
			stack_tw_1:
				"Tailwind CSS je rešitev za dolgoročno uporabo in vzdrževanje CSS-a. Spremljam ga že od prve alfa različice in to spletno mesto dejansko uporablja",
			stack_tw_2:
				". Popolnoma se ujema s Svelte komponentami, povečuje bližino kode in omogoča vizualizacijo še preden aplikacijo poženete.",
			stack_cf_1:
				"Ko stranke iščejo ponudnika, ki bo njihovo kodo izvajal na najbolj varen način, hkrati pa bo izjemno hiter in bo zahteval minimalno vzdrževanja, jim najpogosteje priporočam Cloudflare. Če aplikacija izpolnjuje pogoje za izvajanje na njihovi infrastrukturi, so moja prva izbira. Z obsežno brezplačno ponudbo lahko moje stranke enostavno preizkusijo njihovo storitev in pridobijo zaupanje, medtem ko razvijamo aplikacijo, tako da jo do začetka delovanja že poznajo in jo lahko po potrebi nadgradijo na plačljive nivoje.",
			stack_cf_2: "Mimogrede, to stran postrežejo iz",
			stack_cf_3: "verjetno vam najbližjega.",
			stack_other:
				"Tukaj je še nekaj drugih orodij, ki jih ravno tako uporabljamo in so nam všeč, brez posebnega vrstnega reda",
			stack_disclaimer:
				"Izjava o omejitvi odgovornosti: Nisem povezan, pooblaščen, potrjen ali kakor koli v uradnem odnosu z zgoraj navedenimi projekti ali njihovimi lastniškimi družbami. Njihovi izdelki ali storitve so mi enostavno všeč in jih odkrito priporočam.",
			testimonials_h1: "Kako me<br>**vidijo** stranke",
		},
	};

	return {
		years_of_experience,
		tailwind_version,
		home,
		services: MY_SERVICES,
		testimonials: TESTIMONIALS,
		results_teaser: RESULTS_TEASER,
		products: PRODUCTS,
	};
}
