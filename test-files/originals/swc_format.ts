// Missing spacing around operators
const x=1;
const y=2+3;
let z=a>=b&&c<=d;

// Arrow functions with spacing issues
const fn1=()=>{
	return 42;
};
const fn2=(x:number)=>x+1;
const fn3=async()=>{
	await fetch("/api");
};
const fn4=(x:number,y:string)=>({x,y});

// Single-statement if blocks (expanded — should collapse at 180 width)
if (cond) {
	doSomething();
}
if (a) {
	fa();
}
if (b) {
	fb();
}
for (;;) {
	stmt();
}
while (cond) {
	stmt();
}
if (items[activeIndex]) {
	items[activeIndex].scrollIntoView({
		block: "nearest"
	});
}
if (reallyLongConditionName) {
	reallyLongFunctionCall(withArgs);
}
do {
	stmt();
} while (cond);

// Object literal params (expanded — should collapse)
foo({
	x: 1,
	y: 2,
	z: 3
});
hidden.dispatchEvent(new Event("input", {
	bubbles: true
}));
foo (	{
	key: val
});

// Standalone object literals (expanded — should collapse)
const x = {
	a: 1,
	b: 2
};
return {
	width,
	height
};
return {
	...obj,
	key: val
};

// Array literals (expanded — should collapse)
const arr = [
	"vipsheader",
	safe_path
];
const proc = Bun.spawn([
	"vipsheader",
	safe_path
], {
	windowsHide: true
});

// Inline type literals (expanded by SWC — should collapse back)
function process(config: {
	url: string;
	timeout: number;
	retries?: number;
}) {
	return config;
}
const opts: {
	key: string;
	value: number;
} = { key: "a", value: 1 };
function narrow(props: {
	a: number;
}) {
	return props;
}

// Inline type literal with > max_members — should stay expanded
function wide(props: {
	a: number;
	b: string;
	c: boolean;
	d: string[];
}) {
	return props;
}

// Block comments (JSDoc) that should be preserved
/**
 * Fetches data from the given URL.
 * Returns parsed JSON or throws.
 */
async function fetchData<T>(url: string): Promise<T> {
	const res = await fetch(url);
	return res.json() as T;
}

/* standalone block comment */
const standalone = 1;

// Inline block comment — should stay inline
const inline = 1; /* inline comment */

// Blank lines between interfaces
export interface A {
	x: number;
}

export interface B {
	y: number;
}

// Non-ASCII characters in comments
// Café naïve — ščüéø
const non_ascii = "hello";

// Template literals with nested expressions
const greeting = `Hello, ${name}! You are ${age} years old.`;
const deep = `nested ${`inner ${value}`} end`;

// String concatenation (should be passed through, not SWC's concern)
const msg = "long " +
	"string " +
	"concatenation";

// Type annotations
const items: string[] = [];
const pairs: [string, number][] = [];
const nullable: string | null = null;
