import { greet, ApiResponse } from "./lib.ts";

async function main() {
    const name = (globalThis as any).request.method === "GET" ? "Guest" : "User";
    const greeting = greet(name);
    
    await (Deno as any).core.ops.op_delay(10);
    
    const data: ApiResponse = {
        message: greeting,
        timestamp: Date.now()
    };

    const response = {
        status: 200,
        headers: {
            "content-type": "application/json",
        },
        body: JSON.stringify({
            ...data,
            request: (globalThis as any).request
        })
    };

    (Deno as any).core.ops.op_send_response(response);
}

main();
