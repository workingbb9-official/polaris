export const scriptRunner {
    runNodeScript(userCode, simulation) {
        let output = [];

        const sendPayload = function(payload) {
            try {
                simulation.send_data(payload);
            } catch (e) {
                output.push("API Error: Failed to send payload via Rust.");
            }
        };

        const customConsole = {
            log: (msg) => output.push(msg),
            print: (msg) => output.push(msg)
        };

        try {
            const runner = new Function("sendPayload", "print", "console", userCode);

            runner(sendPayload, customConsole.print, customConsole);

            return { success: true, logs: output };
        } catch (error) {
            return { success: false, logs: [error.message] };
        }
    }
}
