// signals-ui.js — lightweight reactive UI utilities for the static site.
// Self-contained: no external dependencies (no alien-signals).
// Provides: signal, effect, bind_show, bind_collapse, bind_class, bind_text,
//          bind_attr, bind_focus_trap, on_intersect, on_escape, restore_scroll.

(function () {
	"use strict";

	// ── Minimal reactive signal ─────────────────────────────────
	let current_effect = null;

	function signal(value) {
		const subscribers = new Set();
		const s = function (newVal) {
			if (arguments.length === 0) {
				// Getter — register the currently-running effect as a subscriber
				if (current_effect) subscribers.add(current_effect);
				return value;
			}
			// Setter — update value and notify subscribers
			if (value !== newVal) {
				value = newVal;
				const subs = [...subscribers];
				for (const sub of subs) sub();
			}
		};
		return s;
	}

	function effect(fn) {
		const run = () => {
			const prev = current_effect;
			current_effect = run;
			try {
				fn();
			} finally {
				current_effect = prev;
			}
		};
		run();
		return () => {}; // no cleanup needed for our usage
	}

	// ── DOM binding utilities ───────────────────────────────────

	/** Toggle the `hidden` attribute based on a getter signal. */
	function bind_show(el, getter) {
		return effect(() => {
			if (getter()) el.removeAttribute("hidden");
			else el.setAttribute("hidden", "");
		});
	}

	/** Alias: toggle `hidden` attribute (same as bind_show). */
	const bind_collapse = bind_show;

	/** Toggle a CSS class name based on a getter signal. */
	function bind_class(el, getter, name) {
		return effect(() => {
			el.classList.toggle(name, !!getter());
		});
	}

	/** Set textContent based on a getter signal. */
	function bind_text(el, getter) {
		return effect(() => {
			el.textContent = getter();
		});
	}

	/** Set or remove an attribute based on a getter signal. */
	function bind_attr(el, attr, getter) {
		return effect(() => {
			const v = getter();
			if (v === false || v === null || v === undefined) {
				el.removeAttribute(attr);
			} else {
				el.setAttribute(attr, v === true ? "" : String(v));
			}
		});
	}

	/** Trap focus within an element when the getter returns true. */
	function bind_focus_trap(el, getter) {
		let saved_focus = null;
		const handle_tab = (e) => {
			if (e.key !== "Tab") return;
			const focusable = el.querySelectorAll(
				'a, button, input, textarea, select, [tabindex]:not([tabindex="-1"])',
			);
			if (!focusable.length) return;
			const first = focusable[0];
			const last = focusable[focusable.length - 1];
			if (e.shiftKey && document.activeElement === first) {
				e.preventDefault();
				last.focus();
			} else if (!e.shiftKey && document.activeElement === last) {
				e.preventDefault();
				first.focus();
			}
		};
		effect(() => {
			if (getter()) {
				saved_focus = document.activeElement;
				document.body.classList.add("scroll-locked");
				el.addEventListener("keydown", handle_tab);
				queueMicrotask(() => el.querySelector('a, button, input, [tabindex]:not([tabindex="-1"])')?.focus());
			} else {
				document.body.classList.remove("scroll-locked");
				el.removeEventListener("keydown", handle_tab);
				if (saved_focus && document.contains(saved_focus)) {
					saved_focus.focus();
				}
			}
		});
	}

	/** Observe an element for intersection and call setter when visible or when it scrolls above the viewport. */
	function on_intersect(el, setter, opts) {
		const io = new IntersectionObserver(([entry]) => {
			if (entry.isIntersecting || entry.boundingClientRect.top < 0) setter();
		}, opts);
		io.observe(el);
	}

	/** Register a global Escape key handler. */
	function on_escape(handler) {
		window.addEventListener("keydown", (e) => {
			if (e.key === "Escape") handler();
		});
	}

	/** Persist and restore scroll position via sessionStorage. */
	function restore_scroll(el, key) {
		const saved = sessionStorage.getItem(key);
		if (saved !== null) el.scrollTop = parseInt(saved, 10);
		el.addEventListener("scroll", () => sessionStorage.setItem(key, el.scrollTop));
	}

	// ── Export to window ─────────────────────────────────────────
	window.signal = signal;
	window.effect = effect;
	window.bind_show = bind_show;
	window.bind_collapse = bind_collapse;
	window.bind_class = bind_class;
	window.bind_text = bind_text;
	window.bind_attr = bind_attr;
	window.bind_focus_trap = bind_focus_trap;
	window.on_intersect = on_intersect;
	window.on_escape = on_escape;
	window.restore_scroll = restore_scroll;
})();
