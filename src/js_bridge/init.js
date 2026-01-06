import {
    op_log,
    op_send_response,
    op_delay,
    op_req_close,
    op_req_method,
    op_req_path,
    op_req_headers,
    op_req_body,
    op_req_get_header,
    op_sql_execute,
    op_sql_query
} from 'ext:core/ops';

export class Request {
    #rid;

    constructor() {
        this.#rid = globalThis.__JS_REQUEST_RID__;
    }

    method() {
        return op_req_method(this.#rid)
    }

    path() {
        return op_req_path(this.#rid)
    }

    headers() {
        return op_req_headers(this.#rid)
    }

    body() {
        return op_req_body(this.#rid)
    }

    header(k) {
        return op_req_get_header(this.#rid, k)
    }

    close() {
        op_req_close(this.#rid)
    }
}

Object.defineProperty(globalThis, 'request', {
    get() {
        return new Request();
    },
    configurable: true
});

globalThis.db = {
    execute: (sql) => op_sql_execute(sql),
    query: (sql) => op_sql_query(sql),
};