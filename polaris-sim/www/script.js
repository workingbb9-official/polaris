import init, { Simulation } from "../pkg/polaris_sim.js";

let sim = null;
let newCircle = null;
const circles = [];

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
    if (newCircle) {
        return;
    }

    newCircle = document.createElement("div");
    newCircle.classList.add("circle");
    document.body.appendChild(newCircle);

    window.addEventListener("mousemove", moveCircle);
    window.addEventListener("click", dropCircle);
    window.addEventListener("keydown", cancelCircle);
}

function moveCircle(e) {
    if (!newCircle) {
        return;
    }

    newCircle.style.left = `${e.clientX}px`;
    newCircle.style.top = `${e.clientY}px`;
}

function dropCircle(e) {
    if (!newCircle) {
        return;
    }

    if (e.target.tagName == "DIV" || e.target.tagName == "BUTTON") {
        return;
    }

    newCircle.style.backgroundColor = "#cc5500";
    window.removeEventListener("mousemove", moveCircle);
    window.removeEventListener("click", dropCircle);
    window.removeEventListener("keydown", cancelCircle);

    const circle = newCircle;
    circle.dataset.id = circles.length;
    circle.classList.add("node");
    circles.push(circle);

    circle.addEventListener("click", () => displayInfo(circle.dataset.id));

    sim.spawn_node();

    console.log("Node spawned");
    newCircle = null;
}

function cancelCircle(e) {
    if (!newCircle) {
        return;
    }

    if (e.key == "Escape") {
        newCircle.remove();
        window.removeEventListener("mousemove", moveCircle);
        window.removeEventListener("click", dropCircle);
        window.removeEventListener("keydown", cancelCircle);

        newCircle = null;
    }
}

function displayInfo(id) {
    const node = sim.node_info(id);
    const data = JSON.parse(node);
    console.log(`Node ID: ${data.id}`);

    const peers = data.connections;
    if (data.connections.length == 0) {
        console.log("Peers: None");
    } else {
        console.log(`Peers: ${data.connections}`);
    }
}

run();
