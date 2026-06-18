export const nodeRenderer = {
    previewDom: null,
    selectedDom: null,

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

    selectNode(node, id, connections) {
        if (this.selectedDom) {
            this.selectedDom.classList.remove("selected");
        }

        const info = document.getElementById("info-display");
        let html = `<div class="inner-text">ID: ${id}</div>`;

        if (connections.length === 0) {
            html += '<div class="inner-text">Peers: None</div>';
        } else if (connections.length === 1) {
            const peer = connections[0];
            html += `<div class="inner-text">Peer: Node ${peer}</div>`;
        } else {
            const peers = connections.join(", ");
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
