let inject_log = function (path) {
    let conf = { level: 0, count: 0, callback: null };
    let func0 = eval(path);
    if (func0 == undefined) {
        throw `inject_log: ${path} is not defined`;
    }
    if (typeof func0 != "function") {
        throw `inject_log: ${path} is not a function`;
    }

    function func1(...args) {
        if (conf.level >= 1) {
            if (conf.callback) {
                try {
                    conf.callback(this, ...args);
                } catch (e) {
                    msc.log_error(e.toString());
                }
            }
            if (conf.level >= 2) {
                console.groupCollapsed(`[MSC] ${conf.count++} ${path}`);
                console.log("this", this); 1
                console.log("args", args);
                if (conf.level >= 3) {
                    console.trace();
                }
                console.groupEnd();
            }
        } else {
            conf.count = 0;
        }

        return func0.bind(this)(...args);
    };
    eval(`${path} = ${func1}`);
    return conf;
};

// オブジェクトツリーの幅優先探索 (コンソールから使う用)
let search_object = function (
    obj, matcher, max_depth = 8,
    search_type = ["object", "function"],
    exclude_prop = ["_parent"],
    exclude_obj = [window.webkitStorageInfo, window.applicationCache],
) {
    let visited = new Set([obj]);
    let anchor = { depth: 1 };
    let queue = [anchor, [obj, ""/* path */]];

    while (true) {
        let node = queue.shift();
        if (node == anchor) {
            if (queue.length == 0) break;
            if (node.depth > max_depth) break;
            msc.log("the beginning of depth:", node.depth);
            node.depth++;
            queue.push(node);
            continue;
        }

        let obj = node[0], path = node[1];
        for (let key in obj) {
            if (exclude_prop.includes(key)) continue;

            let c_obj = null;
            try {
                c_obj = obj[key];
            } catch (e) {
                msc.log_error(`${path}.${key}`, e);
            }

            if (!search_type.includes(typeof obj)) continue;
            if (exclude_obj.includes(c_obj)) continue;
            if (c_obj instanceof XMLHttpRequest) continue;
            if (visited.has(c_obj)) continue;

            visited.add(c_obj);
            let c_path = path + (isNaN(key) ? `.${key}` : `[${key}]`);
            // msc.log(c_path);
            if (matcher(obj, key)) {
                msc.log(c_path);
            }
            queue.push([c_obj, c_path]);
        }
    }
};

let search_by_object = function (root_obj, target_obj, ...args) {
    function matcher(obj, key) {
        return obj[key] == target_obj;
    }
    msc.search_object(root_obj, matcher, ...args);
};

let search_by_property_name = function (root_obj, prop, ...args) {
    function matcher(obj, key) {
        return key == prop;
    }
    msc.search_object(root_obj, matcher, ...args);
};

let search_by_property_value = function (root_obj, prop, value, ...args) {
    function matcher(obj, key) {
        return key == prop && obj[key] == value;
    }
    msc.search_object(root_obj, matcher, ...args);
};