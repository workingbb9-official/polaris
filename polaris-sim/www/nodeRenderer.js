export const nodeRenderer = {
    previewDom: null,
    selectedDom: null,

    showConfigPage() {
        const info = document.getElementById("info-section");
        info.classList.add("hidden");

        const config = document.getElementById("config-section");
        config.classList.remove("hidden");
    },

    hideConfigPage() {
        const config = document.getElementById("config-section");
        config.classList.add("hidden");

        const info = document.getElementById("info-section");
        info.classList.remove("hidden");
    },

    spawnPreview() {
        const node = document.createElement("div");
        node.classList.add("node", "preview");
        document.body.appendChild(node);
        this.previewDom = node;
    },

    movePreview(clientX, clientY) {
        this.previewDom.style.left = `${clientX}px`;
        this.previewDom.style.top = `${clientY}px`;
    },

    placePreview(id) {
        const node = this.previewDom;
        node.classList.remove("preview");

        node.dataset.id = id;
        this.pendingNode = null;
    },

    removePreview() {
        this.previewDom.remove();
        this.previewDom = null;
    },

    selectNode(node, id, uptime, peers) {
        if (this.selectedDom) {
            this.selectedDom.classList.remove("selected");
        }

        const info = document.getElementById("info-display");
        let html = `<div class="inner-text">ID: ${id}</div>`;
        html += `<div class="inner-text">Uptime: ${uptime}</div>`;

        if (peers.length === 0) {
            html += '<div class="inner-text">Peers: None</div>';
        } else if (peers.length === 1) {
            const peer = peers[0];
            html += `<div class="inner-text">Peer: Node ${peer}</div>`;
        } else {
            const peers = peers.join(", ");
            html += `<div class="inner-text">Peers: Nodes ${peers}</div>`;
        }

        html += '<button id="send" class="btn btn-primary">Send Hello</button>';

        node.classList.add("selected");
        this.selectedDom = node;
        info.innerHTML = html;
    },

    deselectNode() {
        if (this.selectedDom) {
            this.selectedDom.classList.remove("selected");
        }

        this.selectedDom = null;

        const instruction = document.getElementById("select-node");
        instruction.classList.remove("show");

        const info = document.getElementById("info-display");
        info.innerHTML = '<span class="placeholder-text">No node selected</span>';
    },

    startHello() {
        const instruction = document.getElementById("select-node");
        instruction.classList.add("show");
    },

    endHello() {
        const instruction = document.getElementById("select-node");
        instruction.classList.remove("show");
    },
};
