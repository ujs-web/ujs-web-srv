// scripts/db_test.js

async function runTests() {
    try {
        // 1. Setup Table
        await db.execute("CREATE TABLE IF NOT EXISTS js_users (id SERIAL PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL)");
        
        // 2. Insert
        const insertSql = "INSERT INTO js_users (name, email) VALUES ('js_user', 'js@example.com')";
        const affectedRows = await db.execute(insertSql);
        
        // 3. Query
        const querySql = "SELECT name as res1, email as res2 FROM js_users WHERE name = 'js_user'";
        const results = await db.query(querySql);
        
        // 4. Update
        const updateSql = "UPDATE js_users SET email = 'updated@example.com' WHERE name = 'js_user'";
        await db.execute(updateSql);
        
        // 5. Query again to verify update
        const resultsAfterUpdate = await db.query(querySql);
        
        // 6. Delete
        const deleteSql = "DELETE FROM js_users WHERE name = 'js_user'";
        await db.execute(deleteSql);
        
        // 7. Cleanup
        await db.execute("DROP TABLE js_users");

        const response = {
            setup: "ok",
            inserted: affectedRows,
            queried: results,
            updated_email: resultsAfterUpdate[0].res2,
            deleted: "ok"
        };

        Deno.core.ops.op_send_response({
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(response)
        });
    } catch (e) {
        Deno.core.ops.op_send_response({
            status: 500,
            headers: { "Content-Type": "text/plain" },
            body: "Error: " + e.message
        });
    }
}

runTests();
