
export class TestService {
    static getInfo() {
        return "ES Comprehensive Test Service";
    }
}

async function handleRequest() {
    console.log("Starting ES Comprehensive Test...");
    
    // Test async behavior
    await Deno.core.ops.op_delay(20);
    
    const info = TestService.getInfo();
    Deno.core.ops.op_log(`Service info: ${info}`);

    const response = {
        status: 200,
        headers: {
            "content-type": "application/json",
            "x-test-type": "comprehensive-es"
        },
        body: JSON.stringify({
            status: "success",
            service: info,
            request_info: {
                method: globalThis.request.method,
                path: globalThis.request.path
            },
            features_tested: ["ES Classes", "Static Methods", "Async/Await", "JSON Serialization"]
        })
    };

    Deno.core.ops.op_send_response(response);
}

handleRequest();
