import init from './wasm.js';

async function run() {
	await init();

	import("./cloud.js").then((cloud) => {
		cloud.startUp();
	});

}

run();
