import init, { Simulation } from "../pkg/polaris_sim.js";

async function run() {
    await init();
    const sim = new Simulation();
    console.log("Simulation created");

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
