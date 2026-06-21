// Progressive-enhancement scripts for the static Reepolee site.
// (Replaces the SvelteKit reactivity that the original site used.)

(function () {
	"use strict";

	const nav = document.querySelector("[data-site-nav]");
	const menu_toggle = document.querySelector("[data-menu-toggle]");
	const mobile_menu = document.querySelector("[data-mobile-menu]");

	// ── Nav: transparent over the hero at the top; light frosted bar once
	//    scrolled. Over a light hero (data-light-hero, e.g. the homepage) the
	//    text is ink from the start; over the remaining dark heroes it stays
	//    white at the top and flips to ink once the frosted bar kicks in. ──
	const light_hero = !!nav?.hasAttribute("data-light-hero");
	function set_nav_active(active) {
		if (!nav) return;
		nav.classList.toggle("bg-paper/90", active);
		nav.classList.toggle("backdrop-blur-2xl", active);
		nav.classList.toggle("shadow-sm", active);
		const dark_text = active || light_hero;
		nav.classList.toggle("text-ink", dark_text);
		nav.classList.toggle("text-white", !dark_text);
	}
	function update_nav() {
		set_nav_active(window.scrollY !== 0 || menu_open);
	}

	// ── Mobile menu open/close ──
	let menu_open = false;
	function set_menu(open) {
		menu_open = open;
		if (mobile_menu) {
			mobile_menu.classList.toggle("hidden", !open);
			mobile_menu.classList.toggle("flex", open);
			// Focus trap when open, release when closed
			if (window.bind_focus_trap) {
				window.bind_focus_trap(mobile_menu, () => open);
			}
		}
		if (nav) {
			nav.classList.toggle("mobile_navigation_open", open);
			set_nav_active(open || window.scrollY !== 0);
		}
	}

	if (menu_toggle) {
		menu_toggle.addEventListener("click", function (e) {
			e.stopPropagation();
			set_menu(!menu_open);
		});
	}

	// Close the mobile menu when clicking outside it
	document.addEventListener("click", function (e) {
		if (menu_open && nav && !nav.contains(e.target)) set_menu(false);
	});

	// ── Scroll indicator fade ──
	const scroll_indicator = document.querySelector(".scroll-indicator");
	function update_scroll_indicator() {
		if (!scroll_indicator) return;
		scroll_indicator.classList.toggle("fade-out", window.scrollY !== 0);
	}

	window.addEventListener(
		"scroll",
		function () {
			update_nav();
			update_scroll_indicator();
		},
		{ passive: true },
	);
	update_nav();
	update_scroll_indicator();

	// ── Ljubljana clock + working-hours indicator (contact page) ──
	const clock = document.querySelector("[data-ljubljana-time]");
	if (clock) {
		const locale = clock.getAttribute("data-locale") || "en-US";
		const call_label = clock.getAttribute("data-call") || "Call";
		const text_label = clock.getAttribute("data-text") || "Text";

		function update_clock() {
			const now = new Date();
			const time = now.toLocaleTimeString(locale, {
				timeZone: "Europe/Ljubljana",
				hour: "2-digit",
				minute: "2-digit",
			});
			// Working hours: 09:00–17:00 Europe/Ljubljana
			const hour = parseInt(
				new Intl.DateTimeFormat("en-GB", {
					timeZone: "Europe/Ljubljana",
					hour: "2-digit",
					hour12: false,
				}).format(now),
				10,
			);
			const can_call = hour >= 9 && hour < 17;
			const status = can_call
				? "🟢 " + call_label + " 🟢 " + text_label
				: "🔴 " + call_label + " 🟢 " + text_label;
			clock.textContent = time + " " + status;
		}
		update_clock();
		setInterval(update_clock, 30000);
	}
})();
