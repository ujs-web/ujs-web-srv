
async function delay(ms) {
    return Deno.core.ops.op_delay(ms);
}

async function run() {
    console.log("Async script started");
    Deno.core.ops.op_log("Before await");
    
    await delay(1000);
    
    Deno.core.ops.op_log("After await");

    const response = {
        status: 200,
        headers: {
            "content-type": "application/json"
        },
        body: JSON.stringify({
            message: "Hello from async JS",
            path: globalThis.request.path
        })
    };

    Deno.core.ops.op_send_response(response);
}

run();
