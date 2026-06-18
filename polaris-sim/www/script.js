import init, { Simulation } from "../pkg/polaris_sim.js";
import { nodeManager } from "./nodeManager.js";

let sim = null;

const SIM_STATE = {
    DEFAULT: "default",
    SPAWNING: "spawning",
    SELECTED: "selected",
    SENDING_HELLO: "sending hello",
}

const simManager = {
    state: SIM_STATE.DEFAULT,

    handleSpawnClick() {
        if (this.state === SIM_STATE.DEFAULT || this.state === SIM_STATE.SELECTED) {
            nodeManager.spawnPending();
            nodeManager.deselectNode();
            this.state = SIM_STATE.SPAWNING;
        }
    },

    handleInfoClick(e) {
        if (this.state === SIM_STATE.SELECTED && e.target.id === "send") {
            const instruction = document.getElementById("select-node");
            instruction.classList.add("show");
            this.state = SIM_STATE.SENDING_HELLO;
        }
    },

    handleMouseMove(e) {
        if (this.state === SIM_STATE.SPAWNING) {
            nodeManager.movePending(e.clientX, e.clientY);
        }
    },

    handleKeyDown(e) {
        if (this.state === SIM_STATE.SPAWNING && e.key === "Escape") {
            nodeManager.cancelPending();
            this.state = SIM_STATE.DEFAULT;
        }
    },

    handleCanvasClick(e) {
        if (this.state === SIM_STATE.SPAWNING) {
            nodeManager.placePending();
            sim.spawn_node();
            this.state = SIM_STATE.DEFAULT;
        } else if (this.state === SIM_STATE.SELECTED) {
            nodeManager.deselectNode();
            this.state = SIM_STATE.DEFAULT;
        }
    },

    handleDocumentClick(e) {
        const node = e.target.closest(".node");
        if (!node) {
            return;
        }

        switch (this.state) {
            case SIM_STATE.DEFAULT:
            case SIM_STATE.SELECTED:
                const json = sim.node_info(node.dataset.id);
                const data = JSON.parse(json);
                nodeManager.selectNode(node, data.id, data.connections);
                this.state = SIM_STATE.SELECTED;
                break;
            case SIM_STATE.SPAWNING:
                break;
            case SIM_STATE.SENDING_HELLO:
                sendHello(node);
                this.state = SIM_STATE.SELECTED;
                break;
        }

        if (this.state === SIM_STATE.SPAWNING) {
            return;
        }
    },
};

async function run() {
    await init();
    sim = new Simulation();
    initEventListeners();
}

function initEventListeners() {
    const tick = document.getElementById("tick");
    tick.addEventListener("click", () => {
        sim.tick(10);
    });

    const spawn = document.getElementById("spawn");
    spawn.addEventListener("click", () => simManager.handleSpawnClick());

    window.addEventListener("mousemove", (e) => simManager.handleMouseMove(e));
    window.addEventListener("keydown", (e) => simManager.handleKeyDown(e));

    const canvas = document.getElementById("canvas");
    canvas.addEventListener("click", (e) => simManager.handleCanvasClick(e));

    document.body.addEventListener("click", (e) => simManager.handleDocumentClick(e));

    const info = document.getElementById("info-display");
    info.addEventListener("click", (e) => simManager.handleInfoClick(e));
}

function sendHello(to) {
    sim.send_hello(nodeManager.selectedNode.dataset.id, to.dataset.id);
    const instruction = document.getElementById("select-node");
    instruction.classList.remove("show");
}

run();
