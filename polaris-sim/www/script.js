import init, { Simulation } from "../pkg/polaris_sim.js";

let sim = null;
let circle = null;

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
    spawn.addEventListener("click", createCircle);
}

function createCircle() {
    if (circle) {
        return;
    }

    circle = document.createElement("div");
    circle.classList.add("circle");
    document.body.appendChild(circle);

    window.addEventListener("mousemove", moveCircle);
    window.addEventListener("click", dropCircle);
    window.addEventListener("keydown", cancelCircle);
}

function moveCircle(e) {
    if (!circle) {
        return;
    }

    circle.style.left = `${e.clientX}px`;
    circle.style.top = `${e.clientY}px`;
}

function dropCircle(e) {
    if (!circle) {
        return;
    }

    if (e.target.tagName == "DIV" || e.target.tagName == "BUTTON") {
        return;
    }

    circle.style.backgroundColor = "#cc5500";
    window.removeEventListener("mousemove", moveCircle);
    window.removeEventListener("click", dropCircle);
    window.removeEventListener("keydown", cancelCircle);

    console.log("Node spawned");
    sim.spawn_node();
    circle = null;
}

function cancelCircle(e) {
    if (!circle) {
        return;
    }

    if (e.key == "Escape") {
        circle.remove();
        window.removeEventListener("mousemove", moveCircle);
        window.removeEventListener("click", dropCircle);
        window.removeEventListener("keydown", cancelCircle);

        circle = null;
    }
}

run();
