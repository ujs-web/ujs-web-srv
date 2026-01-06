async function handleRequest() {
    try {
        const body = globalThis.request.body();
        console.log('Request body:', body);
        
        const params = JSON.parse(body);
        console.log('Parsed params:', params);
        
        const { name } = params;

        const greeting = name ? `Hello, ${name}!` : 'Hello, World!';
        console.log('Greeting:', greeting);

        Deno.core.ops.op_send_response({
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ greeting })
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
