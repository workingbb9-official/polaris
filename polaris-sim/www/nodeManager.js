export const nodeManager = {
    selectedNode: null,
    pendingNode: null,
    nodes: [],

    spawnPending() {
        if (this.pendingNode) {
            return false;
        }

        const node = document.createElement("div");
        node.classList.add("node", "preview");
        document.body.appendChild(node);
        this.pendingNode = node;

        return true;
    },

    movePending(clientX, clientY) {
        if (!this.pendingNode) {
            return false;
        }

        this.pendingNode.style.left = `${clientX}px`;
        this.pendingNode.style.top = `${clientY}px`;

        return true;
    },

    placePending() {
        if (!this.pendingNode) {
            return false;
        }

        const node = this.pendingNode;
        node.classList.remove("preview");
        node.dataset.id = this.nodes.length;
        this.nodes.push(node);

        this.pendingNode = null;
        return true;
    },

    cancelPending() {
        if (!this.pendingNode) {
            return false;
        }

        this.pendingNode.remove();
        this.pendingNode = null;

        return true;
    },

    selectNode(node, id, connections) {
        if (this.selectedNode) {
            this.selectedNode.classList.remove("selected");
        }

        node.classList.add("selected");

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

        this.selectedNode = node;
        info.innerHTML = html;
    },

    deselectNode() {
        if (!this.selectedNode) {
            return false;
        }

        this.selectedNode.classList.remove("selected");
        this.selectedNode = null;

        const instruction = document.getElementById("select-node");
        instruction.classList.remove("show");

        const info = document.getElementById("info-display");
        info.innerHTML = '<span class="placeholder-text">No node selected</span>';

        return true;
    },
};
