import init, { Simulation } from "../pkg/polaris_sim.js";

let sim = null;

async function run() {
    await init();
    sim = new Simulation();
    initEventListeners();
    console.log("Simulation created");
}

function initEventListeners() {
    const tick = document.getElementById("tick");
    tick.addEventListener("click", () => {
        sim.tick(10);
        console.log(sim.frame());
    });

    const spawn = document.getElementById("spawn");
    spawn.addEventListener("click", () => {
        sim.spawn_node();
        console.log("Node spawned");
    });
}

run();
