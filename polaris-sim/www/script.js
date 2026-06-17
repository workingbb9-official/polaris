import init, { Simulation } from "../pkg/polaris_sim.js";
import { NodeManager } from "./nodeManager.js";

let sim = null;
let isSending = false;
let nodeManager = null;

async function run() {
    await init();
    sim = new Simulation();
    nodeManager = new NodeManager();
    initEventListeners();
}

function initEventListeners() {
    const tick = document.getElementById("tick");
    tick.addEventListener("click", () => {
        console.log("pressed");
        sim.tick(10);
    });

    const spawn = document.getElementById("spawn");
    spawn.addEventListener("click", handleSpawnClick);

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("keydown", handleKeyDown);

    const canvas = document.getElementById("canvas");
    canvas.addEventListener("click", handleCanvasClick);

    document.body.addEventListener("click", handleDocumentClick);

    const info = document.getElementById("info-display");
    info.addEventListener("click", handleInfoClick);
}

function handleSpawnClick() {
    if (nodeManager.spawnPending()) {
        console.log("Spawned");
    }
}

function handleInfoClick(e) {
    if (e.target.id === "send") {
        isSending = true;
        const instruction = document.getElementById("select-node");
        instruction.classList.add("show");
    }

}

function handleMouseMove(e) {
    if (nodeManager.movePending(e.clientX, e.clientY)) {
        console.log("Moving");
    }
}

function handleKeyDown(e) {
    if (e.key === "Escape") {
        if (nodeManager.cancelPending()) {
            console.log("Canceled");
        }
    }
}

function handleCanvasClick() {
    if (nodeManager.placePending()) {
        console.log("Node placed");
        sim.spawn_node();
    }

    if (nodeManager.deselectNode()) {
        console.log("Node deselected");
    }
}

function handleDocumentClick(e) {
    const node = e.target.closest(".node");
    if (!node) {
        return;
    }


    if (isSending) {
        sendHello(node);
        isSending = false;

    } else {
        const json = sim.node_info(node.dataset.id);
        const data = JSON.parse(json);

        nodeManager.selectNode(node, data.id, data.connections);
    }
}

function sendHello(to) {
    sim.send_hello(nodeManager.selectedNode.dataset.id, to.dataset.id);
    isSending = false;
    const instruction = document.getElementById("select-node");
    instruction.classList.remove("show");
}

run();
