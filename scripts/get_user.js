async function handleRequest() {
    try {
        const params = JSON.parse(globalThis.request.body());
        const { id } = params;

        if (!id) {
            throw new Error('User ID is required');
        }

        const users = await db.query(`SELECT * FROM users WHERE id = ${id}`);

        if (users.length === 0) {
            Deno.core.ops.op_send_response({
                status: 404,
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ error: 'User not found' })
            });
            return;
        }

        Deno.core.ops.op_send_response({
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ user: users[0] })
        });
    } catch (error) {
        Deno.core.ops.op_send_response({
            status: 500,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ error: error.message })
        });
    }
}

handleRequest();
