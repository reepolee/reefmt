// Missing spacing
const x=1;
let y=2+3;
var z=a>=b&&c<=d;

// Arrow functions (no parens for single param)
const fn1=x=>x+1;
const fn2=(a,b)=> {
	return a+b;
};
const fn3=async()=>{
	await fetch("/api");
};
const fn4=(x)=>({x});

// if blocks (expanded — should collapse at 180 width)
if (cond) {
	log();
}
for (;;) {
	loop();
}
while (running) {
	tick();
}

// Object literal params (expanded — should collapse)
run({
	mode: "fast",
	debug: false
});
emit("click", {
	bubbles: true
});

// Array literals (expanded — should collapse)
const colors = [
	"red",
	"green",
	"blue"
];
const result = merge([
	"a",
	"b"
], {
	dedupe: true
});

// Block comments preserved
/**
 * Important documentation
 * that spans multiple lines.
 */
function documented() {
	return 42;
}

// Blank lines between functions
function first() {
	return 1;
}

function second() {
	return 2;
}

// Non-ASCII
// Café naïve — ščüéø 😊
const greeting = "Hello, 世界!";

// Template literals
const name = `User: ${user.name} (${user.id})`;
const nested = `outer ${`inner ${val}`} end`;

// String concat
const sql = "SELECT * " +
	"FROM users " +
	"WHERE active = 1";

// Arrow with object return
const factory = (x)=>({
	id: x,
	label: `Item ${x}`,
	active: true
});

// <= and >= should not be affected by arrow spacing
function check(a, b) {
	return a <= b && a >= 0;
}

// Async arrow
const load = async(url)=>{
	const res = await fetch(url);
	return res.json();
};

// Callback with nested arrow
items.map(x=>x.name).filter(n=>n.length>0).forEach(name=>{
	console.log(name);
});

// Immediately-invoked function expression
const result2 = (function() {
	return 42;
})();
