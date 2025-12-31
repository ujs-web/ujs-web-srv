
console.log("Hello from JS");
Deno.core.ops.op_log(`Method: ${globalThis.request.method}, Path: ${globalThis.request.path}`);

const response = {
    status: 200,
    headers: {
        "content-type": "application/json",
        "x-custom": "js-header"
    },
    body: JSON.stringify({
        message: "Hello from JS environment",
        received_body: globalThis.request.body,
        original_path: globalThis.request.path
    })
};

Deno.core.ops.op_send_response(response);
