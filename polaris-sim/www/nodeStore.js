export const SimState = {
    Default: "default",
    Configuring: "configuring",
    Spawning: "spawning",
    Selected: "selected",
    SendingHello: "sending hello",
};

export const nodeStore = {
    state: SimState.Default,
    previewNodeId: null,
    selectedNodeId: null,
    nodes: [],
    currentConfig: {
        heartbeat: null,
    },

    startConfiguring() {
        this.currentConfig.heartbeat = null;
        this.state = SimState.Configuring;
        this.previewNodeId = this.nodes.length;
    },

    cancelConfiguring() {
        this.previewNodeId = null;
        this.state = SimState.Default;
    },

    startSpawning(heartbeat) {
        this.currentConfig.heartbeat = heartbeat;
        this.state = SimState.Spawning;
    },

    createNode(x, y) {
        this.nodes.push({
            id: this.previewNodeId,
            x: x,
            y: y,
        });

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

    getNodeFromId(id) {
        return this.nodes.find(node => node.id === id);
    },
};
