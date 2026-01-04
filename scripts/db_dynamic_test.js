// scripts/db_dynamic_test.js

async function runTests() {
    try {
        // 1. Setup Table
        await db.execute("CREATE TABLE IF NOT EXISTS dynamic_row_test (id SERIAL PRIMARY KEY, name TEXT, age INTEGER, metadata TEXT)");
        
        // 2. Insert
        await db.execute("INSERT INTO dynamic_row_test (name, age, metadata) VALUES ('Alice', 30, 'developer')");
        await db.execute("INSERT INTO dynamic_row_test (name, age, metadata) VALUES ('Bob', 25, 'designer')");
        
        // 3. Dynamic Query - Multiple columns of different types (all cast to text in our current impl)
        // Testing that we don't need 'AS res1' anymore
        const results = await db.query("SELECT id, name, age, metadata FROM dynamic_row_test ORDER BY id");
        
        // 4. Query subset of columns
        const subset = await db.query("SELECT name FROM dynamic_row_test ORDER BY id");

        // 5. Cleanup
        await db.execute("DROP TABLE dynamic_row_test");

        const response = {
            count: results.length,
            first_row: results[0],
            second_row: results[1],
            subset: subset
        };

        Deno.core.ops.op_send_response({
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(response)
        });
    } catch (e) {
        console.log("JS Error: " + e.message + "\nStack: " + e.stack);
        Deno.core.ops.op_send_response({
            status: 500,
            headers: { "Content-Type": "text/plain" },
            body: "Error: " + e.message + "\nStack: " + e.stack
        });
    }
}

runTests();
