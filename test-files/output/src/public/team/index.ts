/**
 * Build-time data loader for the Team page.
 * Ports `partners-data.js`. The reference shuffled partners on each request;
 * for a static build we sort deterministically by last name instead.
 */

const PARTNERS = {
	en: [
		{
			first_name: "Miha",
			last_name: "Medven",
			title: "Team Lead",
			logo: "",
			photo: "/images/responsive/miha-medven.jpg",
			linkedin_url: "https://www.linkedin.com/in/miha-medven/",
			special_url: "https://mihamedven.com/",
			special_caption: "Blog",
			company_url: "https://mihamedven.com/",
			company: "miha-medven",
			slug: "miha-medven",
			content:
				"An effective technical leader with a proven track record building and supporting high performance engineering teams. Worked with over 70 organisations from global market leaders to small garage startups.",
			experience: "15+ years in software development<br>10+ years leadership experience",
		},
		{
			first_name: "Gal",
			last_name: "Jakič",
			title: "Project Manager",
			logo: "",
			photo: "/images/responsive/gal-jakic.jpg",
			linkedin_url: "https://linkedin.com/in/galjakic",
			special_url: "https://gal.jakic.dev/talks",
			special_caption: "Talks",
			company_url: "https://gal.jakic.dev/",
			company: "we-wow-web",
			slug: "gal-jakic",
			content:
				"A web developer turned project manager of fast-growing companies with a passion for building great software development and customer support teams.",
			experience: "10+ Years Experience as Product & Project Manager",
		},
		{
			first_name: "Nik",
			last_name: "Klemenc",
			title: "UX/UI Designer",
			logo: "/assets/partners/nik-klemenc/nik-klemenc-logo.svg",
			photo: "/images/responsive/nik-klemenc.jpg",
			linkedin_url: "https://www.linkedin.com/in/nik-klemenc-7825a9107",
			special_url: "https://www.klemenc.si/",
			special_caption: "Portfolio",
			company_url: "https://www.klemenc.si/",
			company: "nik-klemenc",
			slug: "nik-klemenc",
			content:
				"Creative UX/UI designer and web developer, dedicated to crafting intuitive and engaging digital experiences since 2015. Collaborated with over 40 clients and more than 5 startups. Also, a passionate Svelte enthusiast.",
			experience: "10+ Years Experience in UX/UI Design<br>5+ Years Experience in Front-end development",
		},
		{
			first_name: "Uroš",
			last_name: "Mrak",
			title: "Software Engineer",
			logo: "",
			photo: "/images/responsive/uros-mrak.jpg",
			linkedin_url: "https://www.linkedin.com/in/uroš-mrak-571537116/",
			special_url: "https://uros.space/",
			special_caption: "Web page",
			company_url: "https://uros.space/",
			company: "uros-mrak",
			slug: "uros-mrak",
			content:
				"Software engineer experienced in crafting highly maintainable applications, with a strong background in both front-end and back-end development. Primary focus on front-end engineering, with deep understanding of modern frameworks. I bring a hands-on approach to building scalable and efficient solutions. ",
			experience: "10+ Years Experience as Software Engineer",
		},
		{
			first_name: "Aljaž",
			last_name: "Vaupotič",
			title: "Data & Business Analyst",
			logo: "",
			photo: "/images/responsive/aljaz-vaupotic.jpg",
			linkedin_url: "https://www.linkedin.com/in/alja%C5%BE-vaupoti%C4%8D-97ba0115b",
			special_url: "https://www.bucimap.eu/",
			special_caption: "Web shop",
			company_url: "https://www.bucimap.eu/",
			company: "aljaz-vaupotic",
			slug: "aljaz-vaupotic",
			content:
				"Enthusiastic software developer and analyst who values continuous learning. Enjoys nature and entrepreneurship, constantly exploring new business development opportunities.",
			experience: "5+ Years Experience in Fullstack Software Development & Data Analytics",
		},
		{
			first_name: "Gorazd",
			last_name: "Murnik",
			title: "Digital leader & UX Strategist",
			logo: "",
			photo: "/images/responsive/gorazd-murnik.jpg",
			linkedin_url: "https://www.linkedin.com/in/gorazdmurnik",
			special_url: "https://huggable.be/digital-leader/",
			special_caption: "Personal page",
			company_url: "",
			company: "gorazd-murnik",
			slug: "gorazd-murnik",
			content:
				"Innovative digital leader specializing in UX, digital strategy, and project management. Skilled at optimizing digital platforms and guiding multidisciplinary teams to create impactful, data-driven solutions that drive results.",
			experience: "15+ Years Experience in Digital Product Strategy & UX",
		},
		{
			first_name: "Jure",
			last_name: "Kožuh",
			title: "UX Researcher & Designer",
			logo: "",
			photo: "/images/responsive/jure-kozuh.jpg",
			linkedin_url: "https://www.linkedin.com/in/jurekozuh/",
			special_url: "https://www.kozuh.org/",
			special_caption: "Personal Page",
			company_url: "https://www.kozuh.org/",
			company: "jure-kozuh",
			slug: "jure-kozuh",
			content:
				"UX Designer & Researcher, dedicated to building effective human-machine interactions for a wide range of applications. Passionate about engineering intuitive, functional, and data-driven solutions, with an interest in and deep knowledge of IoT. Worked with clients from a broad range of fields – from medicine to gaming and everything in between – spanning businesses, nonprofits, and startups. NN/g User Experience and IAAP Accessibility certified.",
			experience: "20+ years of experience in user and business-centered design solutions",
		},
	],
	sl: [
		{
			first_name: "Miha",
			last_name: "Medven",
			title: "Team Lead",
			logo: "",
			photo: "/images/responsive/miha-medven.jpg",
			linkedin_url: "https://www.linkedin.com/in/miha-medven/",
			special_url: "https://mihamedven.com/",
			special_caption: "Blog",
			company_url: "https://mihamedven.com/",
			company: "miha-medven",
			slug: "miha-medven",
			content:
				"Učinkovit tehnični vodja z dokazanimi izkušnjami pri oblikovanju in podpiranju visoko zmogljivih inženirskih ekip. Sodeloval je z več kot 70 organizacijami, od vodilnih na svetovnem trgu do majhnih zagonskih podjetij.",
			experience: "15+ let na področju razvoja programske opreme<br>10+ let vodstvenih izkušenj",
		},
		{
			first_name: "Gal",
			last_name: "Jakič",
			title: "Project Manager",
			logo: "",
			photo: "/images/responsive/gal-jakic.jpg",
			linkedin_url: "https://linkedin.com/in/galjakic",
			special_url: "https://gal.jakic.dev/talks",
			special_caption: "Govori",
			company_url: "https://gal.jakic.dev/",
			company: "we-wow-web",
			slug: "gal-jakic",
			content:
				"Spletni razvijalec, ki je postal vodja projektov v hitro rastočih podjetjih s strastjo do oblikovanja odličnih ekip za razvoj programske opreme in podporo strankam.",
			experience: "10+ let izkušenj kot produktni in projektni vodja",
		},
		{
			first_name: "Nik",
			last_name: "Klemenc",
			title: "UX/UI Designer",
			logo: "/assets/partners/nik-klemenc/nik-klemenc-logo.svg",
			photo: "/images/responsive/nik-klemenc.jpg",
			linkedin_url: "https://www.linkedin.com/in/nik-klemenc-7825a9107",
			special_url: "https://www.klemenc.si/",
			special_caption: "Portfelj",
			company_url: "https://www.klemenc.si/",
			company: "nik-klemenc",
			slug: "nik-klemenc",
			content:
				"Kreativni oblikovalec UX/UI in spletni razvijalec, ki od leta 2015 ustvarja intuitivne in privlačne digitalne izkušnje. Sodeloval je z več kot 40 strankami in več kot 5 zagonskimi podjetji. Navdušen Svelte uporabnik.",
			experience: "10+ let izkušenj kot UX/UI oblikovalec<br>5+ let izkušenj kot spletni razvijalec",
		},
		{
			first_name: "Uroš",
			last_name: "Mrak",
			title: "Inženir programske opreme ",
			logo: "",
			photo: "/images/responsive/uros-mrak.jpg",
			linkedin_url: "https://www.linkedin.com/in/uroš-mrak-571537116/",
			special_url: "https://uros.space/",
			special_caption: "Osebna stran",
			company_url: "https://uros.space/",
			company: "uros-mrak",
			slug: "uros-mrak",
			content:
				"Izkušen razvijalec programske opreme, specializiran za razvoj dolgoročno vzdržljivih aplikacij, s poglobljenim znanjem tako na področju front-end kot back-end razvoja. Primarno usmerjen v front-end inženiring z odličnim poznavanjem sodobnih ogrodij. Pri razvoju učinkovitih in razširljivih rešitev se zanašam na praktičen pristop.",
			experience: "10+ let izkušenj kot inženir programske opreme",
		},
		{
			first_name: "Aljaž",
			last_name: "Vaupotič",
			title: "Data & Business Analyst",
			logo: "",
			photo: "/images/responsive/aljaz-vaupotic.jpg",
			linkedin_url: "https://www.linkedin.com/in/alja%C5%BE-vaupoti%C4%8D-97ba0115b",
			special_url: "https://www.bucimap.eu/",
			special_caption: "Spletna trgovina",
			company_url: "https://www.bucimap.eu/",
			company: "aljaz-vaupotic",
			slug: "aljaz-vaupotic",
			content:
				"Navdušen razvijalec programske opreme in analitik, ki ceni trajnostno učenje. Uživa v naravi in podjetništvu ter nenehno raziskuje nove poslovne priložnosti.",
			experience: "5+ leta izkušenj kot razvijalec programske opreme in podatkovni analitik",
		},
		{
			first_name: "Gorazd",
			last_name: "Murnik",
			title: "Digital leader & UX Strategist",
			logo: "",
			photo: "/images/responsive/gorazd-murnik.jpg",
			linkedin_url: "https://www.linkedin.com/in/gorazdmurnik",
			special_url: "https://huggable.be/digital-leader/",
			special_caption: "Osebna stran",
			company_url: "",
			company: "gorazd-murnik",
			slug: "gorazd-murnik",
			content:
				"Vodja digitalnih rešitev specializiran za UX, digitalno strategijo in vodenje projektov. Strokovnjak za optimizacijo digitalnih platform in usmerjanju multidisciplinarnih ekip za ustvarjanje učinkovitih, podatkovno podprtih rešitev, ki prinašajo rezultate.",
			experience: "15+ let izkušenj na področju digitalne produktne strategije in UX-a",
		},
		{
			first_name: "Jure",
			last_name: "Kožuh",
			title: "UX Researcher & Designer",
			logo: "",
			photo: "/images/responsive/jure-kozuh.jpg",
			linkedin_url: "https://www.linkedin.com/in/jurekozuh/",
			special_url: "https://www.kozuh.org/",
			special_caption: "Osebna stran",
			company_url: "https://www.kozuh.org/",
			company: "jure-kozuh",
			slug: "jure-kozuh",
			content:
				"UX oblikovalec in raziskovalec, osredotočen na oblikovanje učinkovitih uporabniških izkušenj za najrazličnejše situacije in okolja. Poudarek namenja razvoju intuitivnih, funkcionalnih in podatkovno podprtih rešitev, z zanimanjem in poglobljenim znanjem o IoT. Sodeloval s strankami iz številnih področij – od medicine do razvijalcev iger ter mnogih drugih – s podjetji, neprofitnimi organizacijami in startupi. NN/g User Experience in IAAP Accessibility certificiran.",
			experience: "20+ let izkušenj pri oblikovanju rešitev, osredotočenih na uporabniške in poslovne potrebe",
		},
	],
};

function sort_by_last_name(list: any[]): any[] {
	const copy = [...list];
	copy.sort((a, b) => a.last_name.localeCompare(b.last_name));
	return copy;
}

export async function load_template_data(): Promise<Record<string, any>> {
	return {
		partners: {
			en: sort_by_last_name(PARTNERS.en),
			sl: sort_by_last_name(PARTNERS.sl),
		},
	};
}
