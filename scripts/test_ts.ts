interface User {
  id: number;
  name: string;
}

const user: User = {
  id: 1,
  name: "Junie"
};

console.log(`[TS Log]: Handling request for ${user.name}`);

const response = {
  status: 200,
  headers: {
    "content-type": "application/json",
    "x-powered-by": "deno-ts-loader"
  },
  body: JSON.stringify({
    message: "Hello from TypeScript!",
    user,
    request: {
      method: (globalThis as any).request.method,
      path: (globalThis as any).request.path
    }
  })
};

(Deno as any).core.ops.op_send_response(response);
