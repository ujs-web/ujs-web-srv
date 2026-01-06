async function handleRequest() {
    try {
        const body = globalThis.request.body();
        console.log('Request body:', body);
        
        const params = JSON.parse(body);
        console.log('Parsed params:', params);
        
        const { a, b } = params;

        if (typeof a !== 'number' || typeof b !== 'number') {
            throw new Error('Parameters must be numbers');
        }

        const result = a + b;
        console.log('Result:', result);

        Deno.core.ops.op_send_response({
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ result })
        });
    } catch (error) {
        console.log('Error:', error.message);
        Deno.core.ops.op_send_response({
            status: 400,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ error: error.message })
        });
    }
}

handleRequest();
