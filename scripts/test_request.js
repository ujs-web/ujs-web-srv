const res = {
    status: 200,
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
        method: request.method(),
        path: request.path(),
        headers: request.headers(),
        body: request.body(),
        x_test: request.header("x-test"),
        non_existent: request.header("non-existent"),
    })
};
Deno.core.ops.op_send_response(res);
request.close();