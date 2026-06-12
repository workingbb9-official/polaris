import init, { Simulation } from "../pkg/polaris_sim.js";

async function run() {
    await init();
    const sim = new Simulation();
    console.log("Simulation created");

    const button = document.getElementById("tick");
    button.addEventListener("click", () => {
        sim.tick(10);
        console.log(sim.frame());
    });
}

run();
