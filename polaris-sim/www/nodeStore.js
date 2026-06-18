export const SimState = {
    Default: "default",
    Spawning: "spawning",
    Selected: "selected",
    SendingHello: "sending hello",
};

export const nodeStore = {
    state: SimState.Default,
    previewNodeId: null,
    selectedNodeId: null,
    nodes: [],

    startSpawning() {
        this.previewNodeId = this.nodes.length;
        this.state = SimState.Spawning;
    },

    createNode() {
        this.nodes.push(this.previewNodeId);
        this.previewNodeId = null;
        this.state = SimState.Default;
    },

    cancelSpawning() {
        this.previewNodeId = null;
        this.state = SimState.Default;
    },

    selectNode(id) {
        this.selectedNodeId = id;
        this.state = SimState.Selected;
    },

    deselectNode() {
        this.selectedNodeId = null;
        this.state = SimState.Default;
    },

    startHello() {
        this.state = SimState.SendingHello;
    },

    endHello() {
        this.state = SimState.Selected;
    },
};
