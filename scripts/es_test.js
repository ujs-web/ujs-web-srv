
export function getMessage() {
    return "Hello from ES Module!";
}

async function run() {
    console.log("ES Module script started");
    
    // 测试 top-level await 支持 (在 ES 模块中可以直接 await，或者在 async 函数中)
    await Deno.core.ops.op_delay(50);
    
    const message = getMessage();
    Deno.core.ops.op_log(message);

    const response = {
        status: 200,
        headers: {
            "content-type": "application/json",
            "x-module-type": "es-module"
        },
        body: JSON.stringify({
            message: message,
            path: globalThis.request.path,
            note: "Successfully loaded and executed as ES Module"
        })
    };

    Deno.core.ops.op_send_response(response);
}

run();
