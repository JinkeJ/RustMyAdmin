/* ================= RustMyAdmin front-end ================= */
"use strict";

const $ = (sel, el) => (el || document).querySelector(sel);
const $$ = (sel, el) => Array.from((el || document).querySelectorAll(sel));

/* ---------- i18n ---------- */
// Language lives in cookie rma_lang (default: en), picked on the login page.
let LANG = (() => {
  const m = document.cookie.match(/(?:^|;\s*)rma_lang=(\w+)/);
  return m && m[1] === "zh" ? "zh" : "en";
})();
function setLang(l) {
  LANG = l;
  document.cookie = "rma_lang=" + l + ";path=/;max-age=31536000;SameSite=Lax";
  document.documentElement.lang = l;
  applyStaticLang();
}
// Translate elements tagged with data-i18n (login page / top bar).
function applyStaticLang() {
  $$("[data-i18n]").forEach((el) => (el.textContent = L(el.dataset.i18n)));
  const sel = $("#li-lang");
  if (sel) sel.value = LANG;
}
const DICT = {
  en: {
    "确定": "OK", "取消": "Cancel", "关闭": "Close", "创建": "Create", "保存": "Save",
    "插入": "Insert", "删除": "Delete", "搜索": "Search", "清除": "Clear", "编辑": "Edit",
    "浏览": "Browse", "结构": "Structure", "操作": "Actions", "加载中...": "Loading...",
    "（无）": "(none)", "连接": "Connect", "连接中...": "Connecting...",
    "未选择文件": "No file selected", "清空": "Clear", "改名": "Rename",
    // errors / api layer
    "会话已失效，请重新连接": "Session expired, please reconnect",
    "未登录": "Not logged in", "请求失败 HTTP {0}": "Request failed: HTTP {0}",
    "未登录或会话已失效": "Not logged in or session expired",
    "主机和用户名不能为空": "Host and username are required",
    "数据库连接已失效": "Database connection lost",
    "内部错误：命令输出类型不匹配": "Internal error: command output type mismatch",
    // sidebar
    "（无表）": "(no tables)", "视图": "View", "表": "Table", "未加载": "Not loaded",
    // home
    "服务器首页": "Server home", "服务器信息": "Server information",
    "当前用户": "Current user", "服务器版本": "Server version",
    "服务器时间": "Server time", "当前数据库": "Current database",
    "新建数据库": "New database", "数据库名": "Database name", "已创建": "Created",
    "SQL 控制台": "SQL console", "在服务器上执行任意 SQL": "Run arbitrary SQL on the server",
    "用户管理": "User management",
    "查看/创建/编辑用户与密码、重载权限": "View/create/edit users & passwords, reload privileges",
    "导入": "Import", "执行 .sql 或 .sql.zip 文件": "Run .sql or .sql.zip files",
    "导出": "Export", "导出数据库为 sql.zip": "Export databases as sql.zip",
    "创建一个新的数据库": "Create a new database",
    // database view
    "数据库 {0}": "Database {0}", "数据库 {0} · 表 {1}": "Database {0} · Table {1}",
    "新建表": "New table", "导出此库": "Export this DB", "导入到此库": "Import into this DB",
    "数据库改名": "Rename database", "删除数据库": "Drop database",
    "名称": "Name", "类型": "Type", "行数(约)": "Rows (approx.)", "大小": "Size",
    "引擎": "Engine", "暂无数据表": "No tables",
    "确认删除": "Confirm deletion", "输入的名称不匹配，已取消": "Name mismatch, cancelled",
    "输入不匹配，已取消": "Input mismatch, cancelled",
    "确定要清空表 {0} 的全部数据吗？此操作不可恢复。": "Empty all data in table {0}? This cannot be undone.",
    "清空表": "Empty table",
    "确定要删除表 {0} 吗？表结构与数据都会被删除，不可恢复。": "Drop table {0}? Structure and data will be removed permanently.",
    "删除表": "Drop table", "将数据库 {0} 重命名为": "Rename database {0} to",
    "确定要把数据库 {0} 重命名为 {1} 吗？": "Rename database {0} to {1}?",
    "将通过 建新库+RENAME TABLE+删旧库 完成，数据不会丢失": "Done via create new DB + RENAME TABLE + drop old DB; data is preserved",
    "请输入数据库名 {0} 以确认": "Type the database name {0} to confirm",
    "确定要删除整个数据库 {0} 吗？其中所有表和数据都会被删除，不可恢复！": "Drop the entire database {0}? All tables and data will be removed permanently!",
    // designer
    "字段名": "Column name", "存储引擎": "Storage engine", "长度/值": "Length/Values",
    "默认值": "Default", "可空": "Nullable", "无符号": "Unsigned", "自增": "A_I",
    "主键": "Primary", "+ 添加字段": "+ Add column", "留空=无": "Empty = none",
    "删除此行": "Remove this row", "至少保留一个字段行": "Keep at least one column row",
    "新建表 · 数据库 {0}": "New table · database {0}", "请填写表名": "Table name is required",
    "字段名不能为空": "Column name is required",
    "字段 {0}（{1}）必须填写长度": "Column {0} ({1}) requires a length",
    "字段 {0}（{1}）需要取值列表，如 'a','b'": "Column {0} ({1}) requires a value list, e.g. 'a','b'",
    "字段 {0}：{1} 不支持 UNSIGNED": "Column {0}: {1} does not support UNSIGNED",
    "字段 {0}：{1} 不支持 AUTO_INCREMENT": "Column {0}: {1} does not support AUTO_INCREMENT",
    "字段 {0}：AUTO_INCREMENT 字段需勾选主键": "Column {0}: AUTO_INCREMENT requires the primary key",
    "字段名重复：{0}": "Duplicate column name: {0}",
    "至少填写一个字段": "At least one column is required",
    "数字": "Numeric", "日期与时间": "Date and time", "字符串": "String",
    "二进制": "Binary", "其他": "Other", "当前类型": "Current type",
    "取值列表，如 'a','b'": "Value list, e.g. 'a','b'", "必填，如 100": "Required, e.g. 100",
    "如 {0}": "e.g. {0}", "不适用": "N/A", "可选": "optional",
    // browse
    "新增一行": "Insert row", "删除选中行": "Delete selected rows",
    "删除选中行（{0}）": "Delete selected rows ({0})", "搜索此表…": "Search this table…",
    "上一页": "Previous", "下一页": "Next", "无数据": "No data",
    "共 {0} 行": "{0} rows total",
    "（此表无主键，行定位使用全部字段，请谨慎操作）": " (no primary key — row targeting uses all columns; proceed with caution)",
    "搜索 “{0}” 找到 {1} 行": "{1} rows found for “{0}”",
    "第 {0} / {1} 页": "Page {0} of {1}", "每页 {0} 行": "Rows per page: {0}",
    "跳至 {0} 页": "Go to page {0}", "删除行": "Delete row", "删除 {0} 行": "Delete {0} rows",
    "确定要删除选中的 {0} 行吗？": "Delete the selected {0} rows?",
    "确定要删除这一行吗？": "Delete this row?", "类型 {0}": "Type {0}", "，可空": ", nullable",
    // row editor
    "编辑行 · {0}.{1}": "Edit row · {0}.{1}", "新增行 · {0}.{1}": "Insert row · {0}.{1}",
    "函数…": "Function…", "函数预设": "Function preset", " · 自增": " · auto inc",
    " · 可从下拉选择": " · dropdown",
    "二进制列：请输入十六进制（可带 0x 前缀）": "Binary column: enter hex (0x prefix optional)",
    "没有修改任何内容": "No changes to save", "没有可插入的字段": "Nothing to insert",
    "自增 ID：{0}": "Last insert ID: {0}",
    // structure page
    "修改": "Change", "上移": "Up", "下移": "Down", "字段结构": "Column structure",
    "主键：{0}": "Primary key: {0}", "（无主键）": "(no primary key)",
    "字段": "Column", "键": "Key", "额外": "Extra", "是": "Yes", "否": "No",
    "新增字段": "Add column", "索引": "Indexes", "创建索引": "Create index",
    "外键": "Foreign keys", "创建外键": "Create foreign key",
    "已是第一列": "Already the first column", "已是最后一列": "Already the last column",
    "删除字段": "Drop column", "唯一": "Unique", "普通": "Index", "暂无索引": "No indexes",
    "方法": "Method", "删除索引": "Drop index", "暂无外键": "No foreign keys",
    "本表字段": "Column", "引用目标": "Reference", "删除外键": "Drop foreign key",
    "确定要删除字段 {0} 吗？该列数据将丢失，不可恢复。": "Drop column {0}? Its data will be lost permanently.",
    "非主键字段不能设置自增。": "Only a primary key column can be AUTO_INCREMENT.",
    "留空=无；可填 NULL": "Empty = none; NULL allowed",
    "修改字段 · {0}.{1}": "Change column · {0}.{1}",
    "{0} 类型必须填写长度": "Type {0} requires a length",
    "{0} 需要取值列表，如 'a','b'": "{0} requires a value list, e.g. 'a','b'",
    "{0} 不支持 UNSIGNED": "{0} does not support UNSIGNED",
    "{0} 不支持 AUTO_INCREMENT": "{0} does not support AUTO_INCREMENT",
    "自增字段必须是主键，当前字段不是主键": "AUTO_INCREMENT requires a primary key; this column is not one",
    "自增字段必须为 NOT NULL": "AUTO_INCREMENT column must be NOT NULL",
    "自增字段不能设置默认值": "AUTO_INCREMENT column cannot have a default",
    // index modal
    "创建索引 · {0}": "Create index · {0}", "索引类型": "Index type",
    "索引名（留空自动生成）": "Index name (auto if empty)",
    "INDEX 普通": "INDEX", "UNIQUE 唯一": "UNIQUE", "FULLTEXT 全文": "FULLTEXT",
    "SPATIAL 空间": "SPATIAL", "PRIMARY 主键": "PRIMARY", "长度": "Length",
    "勾选要包含的字段；“长度”仅前缀索引用（TEXT/BLOB 列必须填），可留空。": "Check the columns to include; Length is only for prefix indexes (required for TEXT/BLOB), optional otherwise.",
    "前缀长度必须是正整数": "Prefix length must be a positive integer",
    "至少勾选一个字段": "Check at least one column",
    // FK modal
    "创建外键 · {0}": "Create foreign key · {0}",
    "约束名（留空自动生成）": "Constraint name (auto if empty)",
    "引用数据库": "Reference database", "引用表": "Reference table",
    "请选择…": "Select…", "默认（NO ACTION）": "Default (NO ACTION)",
    "CASCADE 级联": "CASCADE", "RESTRICT 限制": "RESTRICT",
    "字段配对（{0} → 引用表）": "Column pairs ({0} → referenced table)",
    "+ 添加字段对": "+ Add column pair", "删除此配对": "Remove this pair",
    "至少保留一个字段对": "Keep at least one column pair",
    "请先选择引用表…": "Select the referenced table first…", "加载中…": "Loading…",
    "每一行都必须选择本表字段和引用字段": "Every row needs both the local and referenced column",
    "本表字段不能重复": "Duplicate local column", "引用字段不能重复": "Duplicate referenced column",
    "每一行是一个字段配对；添加多行即创建复合外键（按行顺序对应）。外键需要 InnoDB 等支持外键的引擎。": "Each row pairs a local column with a referenced column; add rows for a composite foreign key (matched in row order). Requires an FK-capable engine such as InnoDB.",
    // operations tab
    "表操作 · {0}": "Table operations · {0}", "常用维护操作。": "Common maintenance operations.",
    "表改名": "Rename table", "用 SQL 修改更多": "More via SQL",
    "导出此表 (sql.zip)": "Export this table (sql.zip)",
    "清空数据（TRUNCATE）": "Empty data (TRUNCATE)", "删除表（DROP）": "Drop table (DROP)",
    "将表 {0} 重命名为": "Rename table {0} to", "请输入表名 {0} 以确认": "Type the table name {0} to confirm",
    // SQL console
    "目标数据库（执行前先 USE）": "Target database (USE before running)",
    "（不切换）": "(no switch)", "出错继续": "Continue on error",
    "在此输入 SQL，支持多语句，Ctrl+Enter 执行": "Enter SQL here; multiple statements supported; Ctrl+Enter to run",
    "执行 (Ctrl+Enter)": "Run (Ctrl+Enter)", "请输入 SQL": "Enter some SQL first",
    "执行中...": "Running…", "（无列）": "(no columns)", "（因出错停止）": " (stopped on error)",
    "返回 {0} 行": "{0} rows returned",
    "（已截断，仅显示前 1000 行）": " (truncated to first 1000 rows)",
    "影响 {0} 行": "{0} rows affected",
    "第 {0} 条语句执行失败：{1}": "Statement {0} failed: {1}",
    "共解析 {0} 条语句，成功 {1} 条": "{0} statements parsed, {1} succeeded",
    // users
    "打开 SQL 控制台": "Open SQL console", "数据库用户": "Database users",
    "创建用户": "Create user", "重载权限 (FLUSH PRIVILEGES)": "Reload privileges (FLUSH PRIVILEGES)",
    "用户名": "Username", "主机": "Host", "认证插件": "Auth plugin", "改密码": "Password",
    "授权": "Privileges", "主机（留空 = localhost）": "Host (empty = localhost)",
    "初始密码": "Initial password", "授权（可选）": "Grants (optional)", "不授权": "No privileges",
    "ALL PRIVILEGES ON *.*（超级用户）": "ALL PRIVILEGES ON *.* (superuser)",
    "ALL PRIVILEGES ON 新库.*（稍后自行 GRANT）": "ALL PRIVILEGES ON a database.* (grant later yourself)",
    "数据库名（选“新库”时填写）": "Database name (for the option above)",
    "用户名不能为空": "Username is required",
    "可以尝试通过 SQL 控制台执行 CREATE USER / GRANT 等语句（若服务器允许）。": "You can try CREATE USER / GRANT statements in the SQL console (if the server allows).",
    "修改密码": "Change password", "使用 ALTER USER ... IDENTIFIED BY": "Uses ALTER USER ... IDENTIFIED BY",
    "用户改名": "Rename user", "删除用户": "Drop user",
    "为用户 {0}@{1} 设置新密码": "Set a new password for {0}@{1}",
    "将用户 {0}@{1} 重命名为": "Rename user {0}@{1} to",
    "确定要删除用户 {0}@{1} 吗？": "Drop user {0}@{1}?",
    "用户 {0}@{1} 已创建": "User {0}@{1} created",
    // grants modal
    "授权 · {0}@{1}": "Privileges · {0}@{1}",
    "当前授权（SHOW GRANTS）": "Current grants (SHOW GRANTS)",
    "作用域": "Scope", "全局 *.*": "Global *.*", "指定数据库": "Database", "指定表": "Table",
    "表名": "Table name", "数据": "Data", "管理": "Administration", "快捷": "Shortcut",
    "ALL PRIVILEGES（全部权限）": "ALL PRIVILEGES (all privileges)",
    "勾选权限后点击“授予”或“收回”，将对该用户在所选作用域执行 GRANT / REVOKE。需要当前账号具有相应权限（GRANT OPTION）。": "Check privileges, then press Grant or Revoke to run GRANT / REVOKE for this user at the chosen scope. Requires the current account to hold GRANT OPTION.",
    "授予勾选权限": "Grant checked", "收回勾选权限": "Revoke checked",
    "请先勾选权限": "Check some privileges first",
    // import / export
    "导入 SQL": "Import SQL", "导入 SQL 文件": "Import SQL file",
    "支持 .sql 文件或包含 .sql 的 .zip（如本工具导出的 xxx.sql.zip）。导入将逐条执行所有语句，遇错停止。": "Supports .sql files and .zip archives containing .sql (such as those exported by this tool). Statements run one by one; execution stops on the first error.",
    "目标数据库（可选；将在导入前 CREATE DATABASE IF NOT EXISTS + USE）": "Target database (optional; CREATE DATABASE IF NOT EXISTS + USE before import)",
    "（不指定，使用文件内的 USE / 库名限定）": "(unspecified — uses USE / qualified names from the file)",
    "选择文件（.sql / .zip）": "Choose file (.sql / .zip)", "开始导入": "Start import",
    "导入中...": "Importing…", "正在上传并执行，请勿关闭页面...": "Uploading and executing — do not close this page…",
    "导入失败": "Import failed",
    "导入中断（部分语句已执行）": "Import interrupted (some statements already executed)",
    "导入完成：成功执行 {0} 条语句": "Import finished: {0} statements executed",
    "导出 SQL": "Export SQL", "导出数据库为 sql.zip": "Export database as sql.zip",
    "数据库": "Database", "导出内容": "Export contents", "结构和数据": "Structure and data",
    "仅结构": "Structure only", "仅数据": "Data only", "导出并下载": "Export and download",
    "选择表（不选 = 全部）": "Choose tables (none = all)", "请选择数据库": "Choose a database",
    "（该库无表）": "(no tables in this database)", "导出失败": "Export failed",
    "导出完成，已开始下载": "Export finished — download started",
    // login page / top bar
    "基于 Rust axum + sqlx 的 MySQL 管理工具": "MySQL administration tool built with Rust axum + sqlx",
    "端口": "Port", "密码": "Password", "语言": "Language", "首页": "Home", "用户": "Users",
    "断开": "Log out",
    "用户名/主机名含非法字符": "Invalid characters in username or host",
    "数据库名含非法字符": "Invalid characters in database name",
    // backend fixed messages
    "已插入 1 行": "1 row inserted", "没有任何修改": "No changes to save",
    "没有任何要插入的字段": "Nothing to insert", "未选择要删除的行": "No rows selected to delete",
    "表不存在": "Table does not exist", "新名称与原名称相同": "The new name is the same as the old one",
    "暂不支持重命名生成列（GENERATED ALWAYS AS）": "Renaming generated columns (GENERATED ALWAYS AS) is not supported",
    "暂不支持移动生成列（GENERATED ALWAYS AS）": "Moving generated columns (GENERATED ALWAYS AS) is not supported",
    "不能移动到自己后面": "Cannot move a column after itself",
    "字段类型不能为空": "Column type is required", "字段类型格式无效": "Invalid column type format",
    "ENUM/SET 需要提供取值列表，如 'a','b'": "ENUM/SET requires a value list, e.g. 'a','b'",
    "取值列表过长": "Value list too long", "取值列表格式无效": "Invalid value list format",
    "长度格式无效（应为数字，如 100 或 8,2）": "Invalid length (expected digits, e.g. 100 or 8,2)",
    "NOT NULL 字段不能用 NULL 作默认值": "NOT NULL columns cannot default to NULL",
    "函数默认值仅支持：DATE→CURRENT_DATE，DATETIME/TIMESTAMP→CURRENT_TIMESTAMP": "Function defaults only support: DATE→CURRENT_DATE, DATETIME/TIMESTAMP→CURRENT_TIMESTAMP",
    "AUTO_INCREMENT 字段必须同时勾选主键": "AUTO_INCREMENT requires the primary key to be checked too",
    "至少需要一个字段": "At least one column is required",
    "字段数量过多（上限 100）": "Too many columns (limit 100)",
    "一张表只能有一个 AUTO_INCREMENT 字段": "A table can have only one AUTO_INCREMENT column",
    "索引类型无效": "Invalid index type", "至少选择一个字段": "Select at least one column",
    "索引字段过多（上限 16）": "Too many index columns (limit 16)",
    "索引前缀长度无效": "Invalid index prefix length",
    "请选择外键字段和引用字段": "Select the FK columns and referenced columns",
    "外键字段与引用字段数量必须一致": "FK columns and referenced columns must match in count",
    "外键字段过多（上限 16）": "Too many FK columns (limit 16)",
    "SQL 不能为空": "SQL is required", "没有可导出的表": "No tables to export",
    "导出内容过大（超过 512MB），请分表导出": "Export too large (over 512MB); export tables in batches",
    "导出模式无效": "Invalid export mode", "zip 中未找到 .sql 文件": "No .sql file found in the zip",
    "未收到文件": "No file received", "搜索内容过长": "Search term too long",
    "此表没有可搜索的字段": "No searchable columns in this table",
    "二进制列请输入十六进制（可带 0x 前缀），例如 4D5A 或 0x4D5A": "Binary columns take hex (0x prefix optional), e.g. 4D5A or 0x4D5A",
    "密码不能为空": "Password is required", "新用户名不能为空": "New username is required",
    "请选择数据库": "Select a database first", "请填写表名": "Enter a table name",
    "作用域无效": "Invalid scope",
    "权限已重新加载（FLUSH PRIVILEGES）": "Privileges reloaded (FLUSH PRIVILEGES)",
    "授权已更新": "Privileges updated", "无法读取服务器状态": "Cannot read server status",
    "行定位信息无效": "Invalid row key data", "二进制主键数据无效": "Invalid binary key data",
    "请勾选要授予的权限": "Check privileges to grant first",
    "请勾选要收回的权限": "Check privileges to revoke first",
  },
};
// Dynamic backend message templates: exact dictionary misses fall through to these regexes.
const DYN = [
  [/^已删除 (\d+) 行$/, "$1 rows deleted"],
  [/^已更新 (\d+) 行$/, "$1 rows updated"],
  [/^表 (.+) 已清空$/, "Table $1 emptied"],
  [/^表 (.+) 已删除$/, "Table $1 dropped"],
  [/^表 (.+) 已创建$/, "Table $1 created"],
  [/^表已重命名为 (.+)$/, "Table renamed to $1"],
  [/^数据库 (.+) 已创建$/, "Database $1 created"],
  [/^数据库 (.+) 已删除$/, "Database $1 dropped"],
  [/^数据库已重命名为 (.+)$/, "Database renamed to $1"],
  [/^字段 (.+) 已添加$/, "Column $1 added"],
  [/^字段 (.+) 已删除$/, "Column $1 dropped"],
  [/^字段 (.+) 已修改$/, "Column $1 changed"],
  [/^字段 (.+) 已修改为 (.+)$/, "Column $1 changed to $2"],
  [/^字段已重命名为 (.+)$/, "Column renamed to $1"],
  [/^字段 (.+) 位置已调整$/, "Column $1 repositioned"],
  [/^索引 (.+) 已创建$/, "Index $1 created"],
  [/^索引 (.+) 已删除$/, "Index $1 dropped"],
  [/^索引 (.+) 不存在$/, "Index $1 does not exist"],
  [/^外键 (.+) 已创建$/, "Foreign key $1 created"],
  [/^外键 (.+) 已删除$/, "Foreign key $1 dropped"],
  [/^外键 (.+) 不存在$/, "Foreign key $1 does not exist"],
  [/^用户 (.+) 已创建$/, "User $1 created"],
  [/^用户 (.+) 已删除$/, "User $1 dropped"],
  [/^用户 (.+) 已重命名为 (.+)$/, "User $1 renamed to $2"],
  [/^用户 (.+) 的密码已修改$/, "Password changed for $1"],
  [/^用户 (.+) 不存在，请先创建用户$/, "User $1 does not exist; create the user first"],
  [/^字段 (.+) 不存在$/, "Column $1 does not exist"],
  [/^表 (.+) 不存在或无权访问$/, "Table $1 does not exist or access is denied"],
  [/^引用表 (.+)\.(.+) 不存在或无权访问$/, "Referenced table $1.$2 does not exist or access is denied"],
  [/^引用字段 (.+) 不存在$/, "Referenced column $1 does not exist"],
  [/^表中已有自增字段 (.+)$/, "Table already has an AUTO_INCREMENT column: $1"],
  [/^字段类型 (.+) 不支持表达式 (.+)$/, "Column type $1 does not support the expression $2"],
  [/^索引字段重复：(.+)$/, "Duplicate index column: $1"],
  [/^外键字段重复：(.+)$/, "Duplicate FK column: $1"],
  [/^引用字段重复：(.+)$/, "Duplicate referenced column: $1"],
  [/^不支持的存储引擎：(.+)$/, "Unsupported storage engine: $1"],
  [/^不支持的字段类型：(.+)$/, "Unsupported column type: $1"],
  [/^不支持的权限：(.+)$/, "Unsupported privilege: $1"],
  [/^外键规则无效：(.+)$/, "Invalid foreign key rule: $1"],
  [/^(.+?)名称不能为空$/, "$1 name is required"],
  [/^SQL 语法错误: ?(.*)$/, "SQL syntax error: $1"],
  [/^连接数据库失败: ?(.*)$/, "Failed to connect to database: $1"],
  [/^数据库连接错误: ?(.*)$/, "Database connection error: $1"],
  [/^无法访问 mysql\.user：(.*)$/, "Cannot access mysql.user: $1"],
  [/^无法读取用户列表（当前账号可能无权访问 mysql\.user）：([\s\S]*)$/, "Cannot read the user list (the current account may lack access to mysql.user): $1"],
  [/^读取上传失败: ?(.*)$/, "Failed to read upload: $1"],
  [/^读取上传文件失败: ?(.*)$/, "Failed to read the uploaded file: $1"],
  [/^打开 zip 失败: ?(.*)$/, "Failed to open zip: $1"],
  [/^读取 zip 条目失败: ?(.*)$/, "Failed to read zip entry: $1"],
  [/^解压失败: ?(.*)$/, "Failed to decompress: $1"],
  [/^写入 zip 失败: ?(.*)$/, "Failed to write zip: $1"],
  [/^完成 zip 失败: ?(.*)$/, "Failed to finalize zip: $1"],
];
/** Translate a UI string / backend message: en mode looks up DICT then DYN; zh returns it as-is. */
const L = (s) => {
  if (LANG !== "en" || typeof s !== "string" || !s) return s;
  if (DICT.en[s]) return DICT.en[s];
  for (const [re, rep] of DYN) {
    const t = s.replace(re, rep);
    if (t !== s) return t;
  }
  return s;
};
/** Translate a template containing {0}, {1}… placeholders. */
const Lf = (s, ...args) => L(s).replace(/\{(\d+)\}/g, (_, i) => String(args[+i]));

const S = {
  token: localStorage.getItem("rma_token") || null,
  info: null,
  dbs: [],
  db: null,
  table: null,
  tab: "browse",
  browse: { page: 1, size: 25, order: null, dir: null, search: null },
  browseMeta: null,
  rowKeys: [],
};

/* ---------- utilities ---------- */
function esc(s) {
  return String(s).replace(/[&<>"']/g, (c) => "&#" + c.charCodeAt(0) + ";");
}
function fmtBytes(n) {
  if (n == null) return "-";
  if (n < 1024) return n + " B";
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + " KB";
  if (n < 1024 * 1024 * 1024) return (n / 1048576).toFixed(1) + " MB";
  return (n / 1073741824).toFixed(2) + " GB";
}

/* ---------- column type list (phpMyAdmin-style groups) ---------- */
const TYPE_GROUPS = [
  ["数字", ["TINYINT", "SMALLINT", "MEDIUMINT", "INT", "BIGINT", "DECIMAL", "FLOAT", "DOUBLE", "BIT"]],
  ["日期与时间", ["DATE", "TIME", "YEAR", "DATETIME", "TIMESTAMP"]],
  ["字符串", ["CHAR", "VARCHAR", "TINYTEXT", "TEXT", "MEDIUMTEXT", "LONGTEXT", "ENUM", "SET"]],
  ["二进制", ["BINARY", "VARBINARY", "TINYBLOB", "BLOB", "MEDIUMBLOB", "LONGBLOB"]],
  ["其他", ["JSON"]],
];
const T_NEED_LEN = new Set(["VARCHAR", "CHAR", "VARBINARY", "BINARY"]); // length required
const T_VAL_LIST = new Set(["ENUM", "SET"]); // length field holds the value list 'a','b'
const T_UNSIGNED = new Set(["TINYINT", "SMALLINT", "MEDIUMINT", "INT", "BIGINT", "DECIMAL", "FLOAT", "DOUBLE"]);
const T_AI = new Set(["TINYINT", "SMALLINT", "MEDIUMINT", "INT", "BIGINT"]); // can auto-increment
const TYPE_LENS = { DECIMAL: "8,2", BIT: "1", YEAR: "4" };

function typeOptions(sel) {
  return TYPE_GROUPS.map(([g, ts]) =>
    `<optgroup label="${L(g)}">` + ts.map((t) => `<option value="${t}"${t === sel ? " selected" : ""}>${t}</option>`).join("") + "</optgroup>"
  ).join("");
}
/** Dynamic placeholder for the Length/Values input. */
function lenPlaceholder(t) {
  if (T_VAL_LIST.has(t)) return L("取值列表，如 'a','b'");
  if (T_NEED_LEN.has(t)) return L("必填，如 100");
  if (TYPE_LENS[t]) return Lf("如 {0}", TYPE_LENS[t]);
  if (t.includes("TEXT") || t.includes("BLOB")) return L("不适用");
  return L("可选");
}
/** Parse an information_schema column type, e.g. "decimal(8,2) unsigned" → { base, length, unsigned } */
function parseColType(typeStr) {
  const m = /^([a-zA-Z]+)(?:\(([\s\S]*)\))?\s*(unsigned)?/.exec(String(typeStr || "").trim());
  if (!m) return { base: "VARCHAR", length: "", unsigned: false };
  return { base: m[1].toUpperCase(), length: m[2] || "", unsigned: !!m[3] };
}
/** Normalize a default value for prefill: strip outer quotes, NULL/implicit empty → "", current_timestamp() → CURRENT_TIMESTAMP */
function parseDefault(d) {
  if (d == null) return "";
  const s = String(d);
  if (/^current_timestamp/i.test(s)) return "CURRENT_TIMESTAMP";
  if (s === "NULL") return "";
  const m = /^'([\s\S]*)'$/.exec(s);
  return m ? m[1].replace(/''/g, "'") : s;
}
/** Type dropdown that also keeps a non-standard current type in its own group. */
function typeOptionsEx(sel) {
  const known = TYPE_GROUPS.some(([, ts]) => ts.includes(sel));
  return (
    (known ? "" : `<optgroup label="${L("当前类型")}"><option value="${esc(sel)}" selected>${esc(sel)}</option></optgroup>`) +
    typeOptions(sel)
  );
}
/** Function presets offered per column type (row insert/edit). */
function colFunctions(col) {
  const t = (col.dataType || "").toUpperCase();
  if (t === "DATETIME" || t === "TIMESTAMP") return ["CURRENT_TIMESTAMP"];
  if (t === "DATE") return ["CURRENT_DATE"];
  if (t === "TIME") return ["CURRENT_TIME"];
  return [];
}
/** Parse the value list out of "enum('a','b')". */
function parseEnumList(typeStr) {
  const m = /^enum\((.*)\)$/i.exec(String(typeStr || "").trim());
  if (!m) return null;
  const out = [];
  const re = /'((?:[^']|'')*)'/g;
  let mm;
  while ((mm = re.exec(m[1])) !== null) out.push(mm[1].replace(/''/g, "'"));
  return out.length ? out : null;
}
function toast(msg, type) {
  const t = document.createElement("div");
  t.className = "toast " + (type === "err" ? "err" : "ok");
  t.textContent = L(msg);
  $("#toast-root").appendChild(t);
  setTimeout(() => t.remove(), 4000);
}
async function api(method, path, body) {
  const headers = { "x-auth-token": S.token || "" };
  let opts = { method, headers };
  if (body !== undefined) {
    headers["content-type"] = "application/json";
    opts.body = JSON.stringify(body);
  }
  const res = await fetch("/api" + path, opts);
  let data = null;
  try { data = await res.json(); } catch (_) { data = {}; }
  if (res.status === 401) {
    showLogin(L("会话已失效，请重新连接"));
    throw new Error(data.error ? L(data.error) : L("未登录"));
  }
  if (!res.ok) throw new Error(data.error ? L(data.error) : Lf("请求失败 HTTP {0}", res.status));
  return data;
}
const apiGet = (p) => api("GET", p);
const apiPost = (p, b) => api("POST", p, b || {});

/* ---------- modals ---------- */
function closeModal() { const m = $("#modal-root"); m.innerHTML = ""; }
function openModal(title, bodyEl, footEl) {
  const root = $("#modal-root");
  root.innerHTML = "";
  const mask = document.createElement("div");
  mask.className = "modal-mask";
  mask.innerHTML = `<div class="modal">
    <div class="modal-head"><span>${esc(L(title))}</span><span class="spacer"></span>
      <button class="close" title="${L("关闭")}">&times;</button></div>
    <div class="modal-body"></div>
    <div class="modal-foot"></div></div>`;
  mask.addEventListener("mousedown", (e) => { if (e.target === mask) closeModal(); });
  $(".modal-body", mask).appendChild(bodyEl);
  if (footEl) $(".modal-foot", mask).appendChild(footEl);
  $(".close", mask).onclick = closeModal;
  root.appendChild(mask);
  const first = $("input, textarea, select", bodyEl);
  if (first) first.focus();
  return mask;
}
/** Modal with a single input (renaming etc.), returns Promise<string|null> */
function promptModal(title, label, initial, hint) {
  return new Promise((resolve) => {
    const body = document.createElement("div");
    body.innerHTML = `<div class="field"><label>${esc(L(label))}</label>
      <input type="text" value="${esc(initial || "")}">
      ${hint ? `<div class="hint">${esc(L(hint))}</div>` : ""}</div>`;
    const foot = document.createElement("div");
    foot.innerHTML = `<button class="btn primary">${L("确定")}</button><button class="btn">${L("取消")}</button>`;
    const mask = openModal(title, body, foot);
    const input = $("input", body);
    const done = (val) => { closeModal(); resolve(val); };
    $(".btn.primary", foot).onclick = () => done(input.value.trim());
    $(".btn:not(.primary)", foot).onclick = () => done(null);
    input.addEventListener("keydown", (e) => { if (e.key === "Enter") done(input.value.trim()); });
    input.focus(); input.select();
  });
}
async function confirmModal(title, text, danger) {
  return new Promise((resolve) => {
    const body = document.createElement("div");
    body.style.cssText = "font-size:13.5px;line-height:1.7;max-width:420px;";
    body.textContent = L(text);
    const foot = document.createElement("div");
    foot.innerHTML = `<button class="btn ${danger ? "danger" : "primary"}">${L("确定")}</button><button class="btn">${L("取消")}</button>`;
    const mask = openModal(title, body, foot);
    $(".btn", foot).onclick = () => { closeModal(); resolve(true); };
    $$(".btn", foot)[1].onclick = () => { closeModal(); resolve(false); };
  });
}

/* ---------- login ---------- */
function showLogin(msg) {
  S.token = null;
  localStorage.removeItem("rma_token");
  $("#app-view").hidden = true;
  $("#login-view").style.display = "flex";
  if (msg) $("#login-error").textContent = msg;
}
async function initApp() {
  try {
    S.info = await apiGet("/status");
  } catch (_) { return; }
  $("#login-view").style.display = "none";
  $("#app-view").hidden = false;
  $("#topinfo").textContent = `${S.info.user} · MySQL ${S.info.version}`;
  await loadSidebar();
  showHome();
}
function bindLogin() {
  const langSel = $("#li-lang");
  langSel.value = LANG;
  langSel.addEventListener("change", () => setLang(langSel.value));
  $("#login-form").addEventListener("submit", async (e) => {
    e.preventDefault();
    const btn = $("#li-btn");
    btn.disabled = true;
    btn.textContent = L("连接中...");
    try {
      const r = await api("POST", "/login", {
        host: $("#li-host").value.trim(),
        port: parseInt($("#li-port").value) || 3306,
        user: $("#li-user").value.trim(),
        password: $("#li-pass").value,
      });
      S.token = r.token;
      localStorage.setItem("rma_token", r.token);
      $("#login-error").textContent = "";
      await initApp();
    } catch (err) {
      $("#login-error").textContent = L(err.message);
    } finally {
      btn.disabled = false;
      btn.textContent = L("连接");
    }
  });
  $("#btn-logout").addEventListener("click", async () => {
    try { await apiPost("/logout"); } catch (_) {}
    showLogin();
  });
}

/* ---------- sidebar ---------- */
async function loadSidebar(keepDb) {
  const el = $("#db-list");
  el.innerHTML = `<div class="side-note">${L("加载中...")}</div>`;
  try {
    const r = await apiGet("/databases");
    S.dbs = r.databases;
  } catch (err) {
    el.innerHTML = `<div class="side-note">${esc(L(err.message))}</div>`;
    return;
  }
  el.innerHTML = "";
  for (const db of S.dbs) {
    const item = document.createElement("div");
    item.className = "db-item";
    item.dataset.db = db;
    item.innerHTML = `<div class="db-head"><span class="arrow">▶</span><span class="dbname">${esc(db)}</span></div><div class="tbl-list"></div>`;
    const head = $(".db-head", item);
    head.addEventListener("click", () => {
      const wasOpen = head.classList.contains("open");
      $$(".db-head.open", el).forEach((h) => h.classList.remove("open"));
      $$(".tbl-list.open", el).forEach((t) => { t.classList.remove("open"); t.innerHTML = ""; });
      if (!wasOpen) {
        head.classList.add("open");
        const list = $(".tbl-list", item);
        list.classList.add("open");
        list.innerHTML = `<div class="side-note">${L("加载中...")}</div>`;
        loadTablesOf(db, list).catch((e) => (list.innerHTML = `<div class="side-note">${esc(L(e.message))}</div>`));
        showDb(db);
      } else {
        head.classList.remove("open");
      }
    });
    el.appendChild(item);
  }
  markSidebar();
}
async function loadTablesOf(db, listEl) {
  const r = await apiGet(`/databases/${encodeURIComponent(db)}/tables`);
  listEl.innerHTML = "";
  if (!r.tables.length) {
    listEl.innerHTML = `<div class="side-note">${L("（无表）")}</div>`;
  }
  for (const t of r.tables) {
    const it = document.createElement("div");
    it.className = "tbl-item";
    it.dataset.table = t.name;
    const label = t.type === "VIEW" ? L("视图") : (t.rows ?? "");
    it.innerHTML = `<span>${esc(t.name)}</span><span class="ttype">${esc(String(label))}</span>`;
    it.title = t.name;
    it.addEventListener("click", () => showTable(db, t.name, "browse"));
    listEl.appendChild(it);
  }
  markSidebar();
}
function markSidebar() {
  $$("#db-list .db-head").forEach((h) => {
    h.classList.toggle("active", h.parentElement.dataset.db === S.db);
  });
  $$("#db-list .tbl-item").forEach((t) => {
    t.classList.toggle("active", t.dataset.table === S.table && t.parentElement.previousElementSibling && t.closest(".db-item").dataset.db === S.db);
  });
}

/* ---------- view switching ---------- */
function setNav(view) {
  S.table = null; S.tab = "browse";
  if (view !== "db") S.db = view === "table" ? S.db : null;
  markSidebar();
}
function crumbs(text) { $("#crumbs").textContent = text || ""; }

async function showHome() {
  setNav("home");
  crumbs(L("服务器首页"));
  S.db = null;
  markSidebar();
  const c = $("#content");
  let st = null;
  try { st = await apiGet("/status"); } catch (_) {}
  const infoHtml = st ? `
    <div class="panel"><h2>${L("服务器信息")}</h2>
      <table class="grid"><tbody>
        <tr><th>${L("当前用户")}</th><td>${esc(st.user)}</td></tr>
        <tr><th>${L("服务器版本")}</th><td>MySQL ${esc(st.version)} ${esc(st.versionComment || "")}</td></tr>
        <tr><th>${L("服务器时间")}</th><td>${esc(st.serverTime || "-")}</td></tr>
        <tr><th>${L("当前数据库")}</th><td>${esc(st.database || L("（无）"))}</td></tr>
      </tbody></table></div>` : "";
  c.innerHTML = `
    ${infoHtml}
    <div class="home-cards">
      <div class="home-card" data-act="newdb"><div class="t">${L("新建数据库")}</div><div class="d">${L("创建一个新的数据库")}</div></div>
      <div class="home-card" data-act="sql"><div class="t">${L("SQL 控制台")}</div><div class="d">${L("在服务器上执行任意 SQL")}</div></div>
      <div class="home-card" data-act="users"><div class="t">${L("用户管理")}</div><div class="d">${L("查看/创建/编辑用户与密码、重载权限")}</div></div>
      <div class="home-card" data-act="import"><div class="t">${L("导入")}</div><div class="d">${L("执行 .sql 或 .sql.zip 文件")}</div></div>
      <div class="home-card" data-act="export"><div class="t">${L("导出")}</div><div class="d">${L("导出数据库为 sql.zip")}</div></div>
    </div>`;
  $$(".home-card", c).forEach((card) => {
    card.addEventListener("click", async () => {
      const act = card.dataset.act;
      if (act === "newdb") {
        const name = await promptModal(L("新建数据库"), L("数据库名"), "");
        if (!name) return;
        try {
          const r = await apiPost("/databases", { name });
          toast(r.message || L("已创建"), "ok");
          await loadSidebar();
          showDb(name);
        } catch (e) { toast(e.message, "err"); }
      } else if (act === "sql") showSql();
      else if (act === "users") showUsers();
      else if (act === "import") showImport();
      else if (act === "export") showExport();
    });
  });
}

/* ---------- database view ---------- */
async function showDb(db) {
  setNav("db");
  S.db = db;
  crumbs(Lf("数据库 {0}", db));
  markSidebar();
  const c = $("#content");
  c.innerHTML = `<div class="panel">${L("加载中...")}</div>`;
  let tables;
  try {
    tables = (await apiGet(`/databases/${encodeURIComponent(db)}/tables`)).tables;
  } catch (e) {
    c.innerHTML = `<div class="panel">${esc(L(e.message))}</div>`;
    return;
  }
  const rows = tables.map((t) => `
    <tr>
      <td><button class="link" data-browse="${esc(t.name)}">${esc(t.name)}</button></td>
      <td>${t.type === "VIEW" ? L("视图") : L("表")}</td>
      <td class="num">${t.rows ?? "-"}</td>
      <td class="num">${fmtBytes(t.sizeBytes)}</td>
      <td>${esc(t.engine || "-")}</td>
      <td class="row-actions">
        <button class="btn sm" data-browse="${esc(t.name)}">${L("浏览")}</button>
        <button class="btn sm" data-struct="${esc(t.name)}">${L("结构")}</button>
        <button class="btn sm danger-ghost" data-empty="${esc(t.name)}">${L("清空")}</button>
        <button class="btn sm danger-ghost" data-dropt="${esc(t.name)}">${L("删除")}</button>
        <button class="btn sm" data-renamet="${esc(t.name)}">${L("改名")}</button>
      </td>
    </tr>`).join("");
  c.innerHTML = `
    <div class="panel">
      <h2>${Lf("数据库 {0}", esc(db))}
        <span class="spacer"></span>
        <button class="btn" id="db-newtable">${L("新建表")}</button>
        <button class="btn" id="db-export">${L("导出此库")}</button>
        <button class="btn" id="db-import">${L("导入到此库")}</button>
        <button class="btn" id="db-rename">${L("数据库改名")}</button>
        <button class="btn danger-ghost" id="db-drop">${L("删除数据库")}</button>
      </h2>
      <div class="grid-wrap"><table class="grid">
        <thead><tr><th>${L("名称")}</th><th>${L("类型")}</th><th class="num">${L("行数(约)")}</th><th class="num">${L("大小")}</th><th>${L("引擎")}</th><th>${L("操作")}</th></tr></thead>
        <tbody>${rows || `<tr><td colspan="6" class="muted">${L("暂无数据表")}</td></tr>`}</tbody>
      </table></div>
    </div>`;
  $$("[data-browse]", c).forEach((b) => (b.onclick = () => showTable(db, b.dataset.browse, "browse")));
  $$("[data-struct]", c).forEach((b) => (b.onclick = () => showTable(db, b.dataset.struct, "structure")));
  $$("[data-renamet]", c).forEach((b) => (b.onclick = () => renameTableFlow(db, b.dataset.renamet, () => showDb(db))));
  $$("[data-empty]", c).forEach((b) => (b.onclick = async () => {
    const t = b.dataset.empty;
    if (!(await confirmModal(L("清空表"), Lf("确定要清空表 {0} 的全部数据吗？此操作不可恢复。", t), true))) return;
    try { const r = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(t)}/truncate`); toast(r.message, "ok"); showDb(db); } catch (e) { toast(e.message, "err"); }
  }));
  $$("[data-dropt]", c).forEach((b) => (b.onclick = async () => {
    const t = b.dataset.dropt;
    if (!(await confirmModal(L("删除表"), Lf("确定要删除表 {0} 吗？表结构与数据都会被删除，不可恢复。", t), true))) return;
    try { const r = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(t)}/drop`); toast(r.message, "ok"); showDb(db); loadSidebar(); } catch (e) { toast(e.message, "err"); }
  }));
  $("#db-newtable").onclick = () => newTableModal(db);
  $("#db-export").onclick = () => showExport(db);
  $("#db-import").onclick = () => showImport(db);
  $("#db-rename").onclick = async () => {
    const to = await promptModal(L("数据库改名"), Lf("将数据库 {0} 重命名为", db), "", L("将通过 建新库+RENAME TABLE+删旧库 完成，数据不会丢失"));
    if (!to) return;
    if (!(await confirmModal(L("数据库改名"), Lf("确定要把数据库 {0} 重命名为 {1} 吗？", db, to), true))) return;
    try {
      const r = await apiPost(`/databases/${encodeURIComponent(db)}/rename`, { to });
      toast(r.message, "ok");
      await loadSidebar();
      showDb(to);
    } catch (e) { toast(e.message, "err"); }
  };
  $("#db-drop").onclick = async () => {
    if (!(await confirmModal(L("删除数据库"), Lf("确定要删除整个数据库 {0} 吗？其中所有表和数据都会被删除，不可恢复！", db), true))) return;
    const name = await promptModal(L("确认删除"), Lf("请输入数据库名 {0} 以确认", db), "");
    if (name !== db) { toast(L("输入的名称不匹配，已取消"), "err"); return; }
    try {
      const r = await apiPost(`/databases/${encodeURIComponent(db)}/drop`);
      toast(r.message, "ok");
      await loadSidebar();
      showHome();
    } catch (e) { toast(e.message, "err"); }
  };
}

/* ---------- new table designer (phpMyAdmin-style) ---------- */
function newTableModal(db) {
  const body = document.createElement("div");
  body.style.cssText = "width:min(780px,92vw);";
  body.innerHTML = `
    <div class="inline-form" style="margin-bottom:14px;">
      <div class="field"><label>${L("表名")}</label><input type="text" id="ct-name" style="width:180px" placeholder="new_table"></div>
      <div class="field"><label>${L("存储引擎")}</label><select id="ct-engine" style="width:120px">
        <option>InnoDB</option><option>MyISAM</option><option>MEMORY</option><option>Aria</option></select></div>
    </div>
    <div class="ct-head">
      <span>${L("字段名")}</span><span>${L("类型")}</span><span>${L("长度/值")}</span><span>${L("默认值")}</span>
      <span class="r">${L("可空")}</span><span class="r">${L("无符号")}</span><span class="r">${L("自增")}</span><span class="r">${L("主键")}</span><span class="r">${L("操作")}</span>
    </div>
    <div id="ct-rows"></div>
    <button class="btn" id="ct-addrow" style="margin-top:10px;">${L("+ 添加字段")}</button>`;
  const rowsEl = $("#ct-rows", body);

  function ctRow(init) {
    init = init || {};
    const r = document.createElement("div");
    r.className = "ct-row";
    r.innerHTML = `
      <input type="text" class="ct-name" placeholder="${L("字段名")}">
      <select class="ct-type">${typeOptions(init.type || "VARCHAR")}</select>
      <input type="text" class="ct-len">
      <input type="text" class="ct-def" placeholder="${L("留空=无")}">
      <label class="ct-chk"><input type="checkbox" class="ct-null"></label>
      <label class="ct-chk"><input type="checkbox" class="ct-uns"></label>
      <label class="ct-chk"><input type="checkbox" class="ct-ai"></label>
      <label class="ct-chk"><input type="checkbox" class="ct-pk"></label>
      <button class="btn sm danger-ghost ct-del" title="${L("删除此行")}">&times;</button>`;
    const tsel = $(".ct-type", r), len = $(".ct-len", r);
    if (init.name) $(".ct-name", r).value = init.name;
    if (init.len) len.value = init.len;
    if (init.nullable) $(".ct-null", r).checked = true;
    if (init.pk) $(".ct-pk", r).checked = true;
    if (init.ai) $(".ct-ai", r).checked = true;
    const syncLen = () => { len.placeholder = lenPlaceholder(tsel.value); };
    tsel.onchange = syncLen;
    syncLen();
    $(".ct-ai", r).onchange = () => {
      if (!$(".ct-ai", r).checked) return;
      if (!T_AI.has(tsel.value)) { tsel.value = "INT"; syncLen(); }
      $(".ct-pk", r).checked = true;
      $(".ct-null", r).checked = false;
    };
    $(".ct-del", r).onclick = () => {
      if ($$(".ct-row", rowsEl).length > 1) r.remove();
      else toast(L("至少保留一个字段行"), "err");
    };
    return r;
  }

  rowsEl.appendChild(ctRow({ type: "INT", name: "id", ai: true, pk: true }));
  rowsEl.appendChild(ctRow({ type: "VARCHAR", len: "100" }));
  $("#ct-addrow", body).onclick = () => rowsEl.appendChild(ctRow({ type: "VARCHAR" }));

  const foot = document.createElement("div");
  foot.innerHTML = `<button class="btn primary">${L("创建")}</button><button class="btn">${L("取消")}</button>`;
  openModal(Lf("新建表 · 数据库 {0}", db), body, foot);
  $("#ct-name", body).focus();

  $(".btn:not(.primary)", foot).onclick = closeModal;
  $(".btn.primary", foot).onclick = async () => {
    const tname = $("#ct-name", body).value.trim();
    if (!tname) { toast(L("请填写表名"), "err"); return; }
    const cols = [];
    const seen = new Set();
    for (const r of $$(".ct-row", rowsEl)) {
      const name = $(".ct-name", r).value.trim();
      const type = $(".ct-type", r).value;
      const len = $(".ct-len", r).value.trim();
      const nullable = $(".ct-null", r).checked;
      const unsigned = $(".ct-uns", r).checked;
      const ai = $(".ct-ai", r).checked;
      const pk = $(".ct-pk", r).checked;
      const def = $(".ct-def", r).value.trim();
      if (!name && !len && !def && !nullable && !unsigned && !ai && !pk) continue; // skip fully empty rows
      if (!name) { toast(L("字段名不能为空"), "err"); return; }
      if (T_NEED_LEN.has(type) && !len) { toast(Lf("字段 {0}（{1}）必须填写长度", name, type), "err"); return; }
      if (T_VAL_LIST.has(type) && !len) { toast(Lf("字段 {0}（{1}）需要取值列表，如 'a','b'", name, type), "err"); return; }
      if (unsigned && !T_UNSIGNED.has(type)) { toast(Lf("字段 {0}：{1} 不支持 UNSIGNED", name, type), "err"); return; }
      if (ai && !T_AI.has(type)) { toast(Lf("字段 {0}：{1} 不支持 AUTO_INCREMENT", name, type), "err"); return; }
      if (ai && !pk) { toast(Lf("字段 {0}：AUTO_INCREMENT 字段需勾选主键", name), "err"); return; }
      const lk = name.toLowerCase();
      if (seen.has(lk)) { toast(Lf("字段名重复：{0}", name), "err"); return; }
      seen.add(lk);
      cols.push({ name, type, length: len, nullable, unsigned, auto_increment: ai, primary_key: pk, default: def || null });
    }
    if (!cols.length) { toast(L("至少填写一个字段"), "err"); return; }
    try {
      const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables`, {
        name: tname,
        columns: cols,
        engine: $("#ct-engine", body).value,
      });
      toast(res.message, "ok");
      closeModal();
      await loadSidebar();
      showTable(db, tname, "structure");
    } catch (e) { toast(e.message, "err"); }
  };
}

/* ---------- table view (tabs) ---------- */
async function showTable(db, table, tab) {
  setNav("db");
  // reset browse state (page/sort/search) when switching to another db/table
  if (S.db !== db || S.table !== table) {
    S.browse = { page: 1, size: 25, order: null, dir: null, search: null };
  }
  S.db = db; S.table = table; S.tab = tab || "browse";
  crumbs(Lf("数据库 {0} · 表 {1}", db, table));
  markSidebar();
  const c = $("#content");
  c.innerHTML = `
    <div class="tabs">
      <button class="tab" data-tab="browse">${L("浏览")}</button>
      <button class="tab" data-tab="structure">${L("结构")}</button>
      <button class="tab" data-tab="sql">SQL</button>
      <button class="tab" data-tab="operations">${L("操作")}</button>
    </div>
    <div id="tab-content"></div>`;
  $$(".tab", c).forEach((b) => {
    b.classList.toggle("active", b.dataset.tab === S.tab);
    b.onclick = () => showTable(db, table, b.dataset.tab);
  });
  const tc = $("#tab-content");
  if (S.tab === "browse") await renderBrowse(db, table, tc);
  else if (S.tab === "structure") await renderStructure(db, table, tc);
  else if (S.tab === "sql") renderSqlTab(db, table, tc);
  else if (S.tab === "operations") renderOperations(db, table, tc);
}

/* ---------- data browse ---------- */
async function renderBrowse(db, table, tc) {
  const q = S.browse;
  tc.innerHTML = `<div class="panel">${L("加载中...")}</div>`;
  let r;
  try {
    const p = new URLSearchParams({ page: q.page, size: q.size });
    if (q.order) { p.set("order", q.order); p.set("dir", q.dir || "asc"); }
    if (q.search) p.set("search", q.search);
    r = await apiGet(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/rows?` + p);
  } catch (e) { tc.innerHTML = `<div class="panel">${esc(L(e.message))}</div>`; return; }

  S.browseMeta = r;
  S.rowKeys = r.rows.map((x) => x.k);
  const cols = r.columns;

  const headCells = cols.map((c1) => {
    const mark = q.order === c1.name ? `<span class="sortmark">${q.dir === "desc" ? "▼" : "▲"}</span>` : "";
    const pkIcon = c1.key === "PRI" ? " 🔑" : "";
    const tip = Lf("类型 {0}", c1.type) + (c1.nullable ? L("，可空") : "");
    return `<th class="sortable" data-col="${esc(c1.name)}" title="${esc(tip)}">${esc(c1.name)}${pkIcon} ${mark}</th>`;
  }).join("");

  const bodyRows = r.rows.map((row, ri) => {
    const cells = row.c.map((v, ci) => {
      if (v === null) return `<td class="null">NULL</td>`;
      const bin = cols[ci] && cols[ci].binary;
      if (bin) return `<td class="bin">${esc(v)}</td>`;
      const long = v.length > 100;
      return `<td class="${long ? "cell-long" : ""}" ${long ? `title="${esc(v)}"` : ""}>${esc(v)}</td>`;
    }).join("");
    return `<tr data-ri="${ri}"><td><input type="checkbox" class="row-check"></td>${cells}
      <td class="row-actions"><button class="btn sm" data-edit="${ri}">${L("编辑")}</button>
      <button class="btn sm danger-ghost" data-del="${ri}">${L("删除")}</button></td></tr>`;
  }).join("");

  const pageSel = [25, 50, 100, 200].map((n) => `<option ${n === r.size ? "selected" : ""}>${n}</option>`).join("");
  const pager = `
      <div class="pager">
        <button class="btn sm" data-pg="first" ${r.page <= 1 ? "disabled" : ""}>«</button>
        <button class="btn sm" data-pg="prev" ${r.page <= 1 ? "disabled" : ""}>${L("上一页")}</button>
        <span>${Lf("第 {0} / {1} 页", r.page, r.pages)}</span>
        <button class="btn sm" data-pg="next" ${r.page >= r.pages ? "disabled" : ""}>${L("下一页")}</button>
        <button class="btn sm" data-pg="last" ${r.page >= r.pages ? "disabled" : ""}>»</button>
        <span>${Lf("每页 {0} 行", `<select class="pg-size">${pageSel}</select>`)}</span>
        <span>${Lf("跳至 {0} 页", `<input type="number" class="pg-jump" min="1" max="${r.pages}" style="width:70px;padding:3px 6px;border:1px solid var(--border);border-radius:6px" value="${r.page}">`)}</span>
      </div>`;
  const statusLine = q.search
    ? Lf("搜索 “{0}” 找到 {1} 行", q.search, r.total)
    : Lf("共 {0} 行", r.total) + (r.primaryKey.length ? "" : L("（此表无主键，行定位使用全部字段，请谨慎操作）"));
  tc.innerHTML = `
    <div class="panel">
      <div class="toolbar">
        <button class="btn primary" id="br-insert">${L("新增一行")}</button>
        <button class="btn danger" id="br-del-sel" disabled>${L("删除选中行")}</button>
        <span class="spacer"></span>
        <input type="text" id="br-search" placeholder="${L("搜索此表…")}" style="width:180px;padding:6px 10px;border:1px solid var(--border);border-radius:6px;font-size:12.5px" value="${esc(q.search || "")}">
        <button class="btn" id="br-search-go">${L("搜索")}</button>
        ${q.search ? `<button class="btn" id="br-search-clear">${L("清除")}</button>` : ""}
        <span class="muted">${statusLine}</span>
      </div>
      ${pager}
      <div class="grid-wrap"><table class="grid">
        <thead><tr><th><input type="checkbox" id="check-all"></th>${headCells}<th>${L("操作")}</th></tr></thead>
        <tbody>${bodyRows || `<tr><td colspan="${cols.length + 2}" class="muted">${L("无数据")}</td></tr>`}</tbody>
      </table></div>
      ${pager}
    </div>`;

  const refresh = () => renderBrowse(db, table, tc);
  $$("th.sortable", tc).forEach((th) => (th.onclick = () => {
    const col = th.dataset.col;
    if (q.order === col) q.dir = q.dir === "desc" ? "asc" : "desc";
    else { q.order = col; q.dir = "asc"; }
    q.page = 1;
    refresh();
  }));
  $("#check-all").onclick = (e) => $$(".row-check", tc).forEach((ck) => (ck.checked = e.target.checked));
  $$(".row-check", tc).forEach((ck) => (ck.onchange = () => {
    const sel = $$(".row-check", tc).filter((x) => x.checked).length;
    $("#br-del-sel").disabled = sel === 0;
    $("#br-del-sel").textContent = sel ? Lf("删除选中行（{0}）", sel) : L("删除选中行");
  }));
  $$("[data-del]", tc).forEach((b) => (b.onclick = async () => {
    const ri = parseInt(b.dataset.del);
    if (!(await confirmModal(L("删除行"), L("确定要删除这一行吗？"), true))) return;
    try {
      const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/rows/delete`, { keys: [S.rowKeys[ri]] });
      toast(res.message, "ok");
      refresh();
    } catch (e) { toast(e.message, "err"); }
  }));
  $$("[data-edit]", tc).forEach((b) => (b.onclick = () => editRowModal(db, table, parseInt(b.dataset.edit), refresh)));
  $("#br-insert").onclick = () => insertRowModal(db, table, refresh);
  $("#br-del-sel").onclick = async () => {
    const keys = $$(".row-check", tc).filter((ck) => ck.checked).map((ck) => S.rowKeys[parseInt(ck.closest("tr").dataset.ri)]);
    if (!keys.length || !(await confirmModal(Lf("删除 {0} 行", keys.length), Lf("确定要删除选中的 {0} 行吗？", keys.length), true))) return;
    try {
      const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/rows/delete`, { keys });
      toast(res.message, "ok");
      refresh();
    } catch (e) { toast(e.message, "err"); }
  };
  $$("[data-pg]", tc).forEach((b) => (b.onclick = () => {
    const act = b.dataset.pg;
    if (act === "first") q.page = 1;
    else if (act === "prev") q.page = Math.max(1, q.page - 1);
    else if (act === "next") q.page = Math.min(r.pages, q.page + 1);
    else q.page = r.pages;
    refresh();
  }));
  $$(".pg-size", tc).forEach((s) => (s.onchange = (e) => { q.size = parseInt(e.target.value); q.page = 1; refresh(); }));
  $$(".pg-jump", tc).forEach((i) => (i.onchange = (e) => { q.page = Math.min(r.pages, Math.max(1, parseInt(e.target.value) || 1)); refresh(); }));
  // quick search
  $("#br-search-go", tc).onclick = () => { q.search = $("#br-search", tc).value.trim() || null; q.page = 1; refresh(); };
  $("#br-search", tc).addEventListener("keydown", (e) => { if (e.key === "Enter") $("#br-search-go", tc).click(); });
  const searchClear = $("#br-search-clear", tc);
  if (searchClear) searchClear.onclick = () => { q.search = null; q.page = 1; refresh(); };
}

/* ---------- row edit / insert modal ---------- */
function buildRowForm(columns, values) {
  const wrap = document.createElement("div");
  wrap.className = "edit-row-grid";
  // wide enough that the NULL checkbox sits next to the input without horizontal scrolling
  wrap.style.cssText = "width:min(760px,94vw);";
  for (const col of columns) {
    const isAuto = /auto_increment/i.test(col.extra || "");
    const cell = document.createElement("div");
    cell.className = "edit-cell";
    let cur = values ? values[col.name] : undefined;
    let showVal = "";
    let showNull = false;
    let presetFn = "";
    const fnList = colFunctions(col);
    if (values) {
      showNull = cur === null;
      if (cur !== null && cur !== undefined) showVal = cur;
    } else {
      // new row: prefill column defaults (strip quotes/NULL; function defaults select the preset directly)
      const dv = parseDefault(col.default);
      if (fnList.includes(dv)) {
        presetFn = dv;
      } else {
        showVal = dv;
      }
      showNull = col.default == null && col.nullable;
    }
    const enumVals = (col.dataType || "").toLowerCase() === "enum" ? parseEnumList(col.type) : null;
    const longText = !enumVals && (/text|blob|binary|json/i.test(col.dataType) || String(showVal).length > 80);
    const inputTag = longText ? "textarea" : 'input type="text"';
    const fnSel = fnList.length
      ? `<select class="cell-fn" title="${L("函数预设")}"><option value="">${L("函数…")}</option>${fnList.map((f) => `<option value="${f}"${presetFn === f ? " selected" : ""}>${f}</option>`).join("")}</select>`
      : "";
    const valCtrl = enumVals
      ? `<select class="cell-inp"><option value=""></option>${enumVals.map((v) => `<option value="${esc(v)}"${String(showVal) === v ? " selected" : ""}>${esc(v)}</option>`).join("")}</select>`
      : `<${inputTag} class="cell-inp" ${isAuto && values ? "readonly" : ""}>${longText ? esc(String(showVal).replace(/^\s+/, "")) : ""}</${longText ? "textarea" : "input"}>`;
    cell.innerHTML = `
      <div class="colname">${esc(col.name)}<span class="ctype">${esc(col.type)}${isAuto ? L(" · 自增") : ""}${enumVals ? L(" · 可从下拉选择") : ""}</span></div>
      <div class="val-wrap">${fnSel}${valCtrl}</div>
      <div class="null-box"><label><input type="checkbox" class="cell-null" ${showNull ? "checked" : ""}>NULL</label></div>`;
    const inp = $(".cell-inp", cell);
    const fnBox = $(".cell-fn", cell);
    if (!longText && !enumVals) inp.value = showVal;
    const syncDisabled = () => {
      inp.disabled = $(".cell-null", cell).checked || (fnBox ? !!fnBox.value : false);
    };
    if (fnBox) fnBox.addEventListener("change", syncDisabled);
    $(".cell-null", cell).addEventListener("change", syncDisabled);
    syncDisabled();
    if (col.binary) {
      if (String(inp.value).startsWith("0x")) inp.value = inp.value.slice(2);
      const hint = document.createElement("div");
      hint.className = "hint";
      hint.style.cssText = "grid-column:2;font-size:11px;color:#94a3b8";
      hint.textContent = L("二进制列：请输入十六进制（可带 0x 前缀）");
      cell.appendChild(hint);
    }
    wrap.appendChild(cell);
  }
  return wrap;
}
function readRowForm(wrap, columns) {
  const out = [];
  const cells = $$(".edit-cell", wrap);
  for (let i = 0; i < cells.length; i++) {
    const col = columns[i];
    const isAuto = /auto_increment/i.test(col.extra || "");
    const nullBox = $(".cell-null", cells[i]);
    const inp = $(".cell-inp", cells[i]);
    const fnBox = $(".cell-fn", cells[i]);
    if (nullBox.checked) {
      out.push({ col: col.name, value: "", is_null: true, is_expr: false });
    } else if (fnBox && fnBox.value) {
      out.push({ col: col.name, value: fnBox.value, is_null: false, is_expr: true });
    } else {
      let v = inp.value;
      if (col.binary && v.trim().startsWith("0x")) v = v.trim().slice(2);
      out.push({ col: col.name, value: v, is_null: false, is_expr: false });
    }
    out[out.length - 1]._auto = isAuto;
  }
  return out;
}
function editRowModal(db, table, ri, onDone) {
  const meta = S.browseMeta;
  const row = meta.rows[ri];
  const values = {};
  meta.columns.forEach((c, i) => (values[c.name] = row.c[i]));
  const wrap = buildRowForm(meta.columns, values);
  // keep original values for change detection
  const origValues = {};
  meta.columns.forEach((c, i) => {
    let v = row.c[i];
    if (v !== null && c.binary && typeof v === "string" && v.startsWith("0x")) v = v.slice(2);
    origValues[c.name] = v;
  });
  const foot = document.createElement("div");
  foot.innerHTML = `<button class="btn primary">${L("保存")}</button><button class="btn">${L("取消")}</button>`;
  openModal(Lf("编辑行 · {0}.{1}", db, table), wrap, foot);
  $(".btn.primary", foot).onclick = async () => {
    const changesAll = readRowForm(wrap, meta.columns);
    // submit only fields that changed (or switched to/from NULL)
    const changes = changesAll.filter((ch) => {
      const o = origValues[ch.col];
      if (ch.is_null) return o !== null;
      return o !== ch.value;
    });
    if (!changes.length) { toast(L("没有修改任何内容")); closeModal(); return; }
    try {
      const r = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/rows/update`, {
        key: S.rowKeys[ri],
        changes: changes.map((c) => ({ col: c.col, value: c.value, is_null: c.is_null, is_expr: c.is_expr })),
      });
      toast(r.message, "ok");
      closeModal();
      onDone && onDone();
    } catch (e) { toast(e.message, "err"); }
  };
  $(".btn:not(.primary)", foot).onclick = closeModal;
}
function insertRowModal(db, table, onDone) {
  const meta = S.browseMeta;
  if (!meta) return;
  const wrap = buildRowForm(meta.columns, null);
  const foot = document.createElement("div");
  foot.innerHTML = `<button class="btn primary">${L("插入")}</button><button class="btn">${L("取消")}</button>`;
  openModal(Lf("新增行 · {0}.{1}", db, table), wrap, foot);
  $(".btn.primary", foot).onclick = async () => {
    const valuesAll = readRowForm(wrap, meta.columns);
    // skip empty auto-increment / generated fields
    const values = valuesAll.filter((v) => !(v._auto && !v.is_null && v.value === ""));
    if (!values.length) { toast(L("没有可插入的字段")); return; }
    try {
      const r = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/rows/insert`, {
        values: values.map((v) => ({ col: v.col, value: v.value, is_null: v.is_null, is_expr: v.is_expr })),
      });
      toast(r.message, "ok");
      closeModal();
      onDone && onDone();
    } catch (e) { toast(e.message, "err"); }
  };
  $(".btn:not(.primary)", foot).onclick = closeModal;
}

/* ---------- table structure ---------- */
/** Change-column modal (rename + type/length/nullable/unsigned/default/auto-increment) */
function modifyColumnModal(db, table, meta, pkArr, refresh) {
  const cur = parseColType(meta.type);
  const isPk = (pkArr || []).includes(meta.name);
  const isAi = /auto_increment/i.test(meta.extra || "");
  const body = document.createElement("div");
  body.style.cssText = "width:min(560px,92vw);";
  body.innerHTML = `
    <div class="inline-form" style="align-items:flex-start;">
      <div class="field"><label>${L("字段名")}</label><input type="text" id="mc-name" style="width:150px" value="${esc(meta.name)}"></div>
      <div class="field"><label>${L("类型")}</label><select id="mc-type" style="width:130px">${typeOptionsEx(cur.base)}</select></div>
      <div class="field"><label>${L("长度/值")}</label><input type="text" id="mc-len" style="width:150px" value="${esc(cur.length)}"></div>
    </div>
    <div class="inline-form" style="margin-top:10px;align-items:flex-start;">
      <div class="field"><label>${L("可空")}</label><label class="ct-chk"><input type="checkbox" id="mc-null"${meta.nullable ? " checked" : ""}></label></div>
      <div class="field"><label>${L("无符号")}</label><label class="ct-chk"><input type="checkbox" id="mc-unsigned"${cur.unsigned ? " checked" : ""}></label></div>
      <div class="field"><label>${L("自增")}</label><label class="ct-chk"><input type="checkbox" id="mc-ai"${isAi ? " checked" : ""}></label></div>
      <div class="field"><label>${L("默认值")}</label><input type="text" id="mc-default" style="width:170px" value="${esc(parseDefault(meta.default))}" placeholder="${L("留空=无；可填 NULL")}"></div>
    </div>
    <div class="hint" style="margin-top:10px;">${Lf("主键：{0}。", isPk ? L("是") : L("否"))}${isPk ? "" : L("非主键字段不能设置自增。")}</div>`;
  const foot = document.createElement("div");
  foot.innerHTML = `<button class="btn primary">${L("保存")}</button><button class="btn">${L("取消")}</button>`;
  openModal(Lf("修改字段 · {0}.{1}", table, meta.name), body, foot);

  const syncLen = () => { $("#mc-len", body).placeholder = lenPlaceholder($("#mc-type", body).value); };
  $("#mc-type", body).onchange = syncLen;
  syncLen();

  $(".btn:not(.primary)", foot).onclick = closeModal;
  $(".btn.primary", foot).onclick = async () => {
    const name = $("#mc-name", body).value.trim();
    const type = $("#mc-type", body).value;
    const len = $("#mc-len", body).value.trim();
    const nullable = $("#mc-null", body).checked;
    const unsigned = $("#mc-unsigned", body).checked;
    const ai = $("#mc-ai", body).checked;
    const def = $("#mc-default", body).value.trim();
    if (!name) { toast(L("字段名不能为空"), "err"); return; }
    if (T_NEED_LEN.has(type) && !len) { toast(Lf("{0} 类型必须填写长度", type), "err"); return; }
    if (T_VAL_LIST.has(type) && !len) { toast(Lf("{0} 需要取值列表，如 'a','b'", type), "err"); return; }
    if (unsigned && !T_UNSIGNED.has(type)) { toast(Lf("{0} 不支持 UNSIGNED", type), "err"); return; }
    if (ai && !T_AI.has(type)) { toast(Lf("{0} 不支持 AUTO_INCREMENT", type), "err"); return; }
    if (ai && !isPk) { toast(L("自增字段必须是主键，当前字段不是主键"), "err"); return; }
    try {
      const res = await apiPost(
        `/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/columns/${encodeURIComponent(meta.name)}/modify`,
        { name, type, length: len, nullable, unsigned, auto_increment: ai, default: def || null }
      );
      toast(res.message, "ok");
      closeModal();
      refresh();
    } catch (e) { toast(e.message, "err"); }
  };
}

/** Create-index modal (kind + name + multi-column + optional prefix length) */
function addIndexModal(db, table, columns, refresh, reloadIdx) {
  const KINDS = [
    ["index", "INDEX 普通"],
    ["unique", "UNIQUE 唯一"],
    ["fulltext", "FULLTEXT 全文"],
    ["spatial", "SPATIAL 空间"],
    ["primary", "PRIMARY 主键"],
  ];
  const body = document.createElement("div");
  body.style.cssText = "width:min(520px,92vw);";
  body.innerHTML = `
    <div class="inline-form" style="margin-bottom:10px;">
      <div class="field"><label>${L("索引类型")}</label><select id="ix-kind" style="width:160px">
        ${KINDS.map(([v, t]) => `<option value="${v}">${L(t)}</option>`).join("")}</select></div>
      <div class="field"><label>${L("索引名（留空自动生成）")}</label><input type="text" id="ix-name" style="width:220px"></div>
    </div>
    <div class="hint" style="margin-bottom:8px;">${L("勾选要包含的字段；“长度”仅前缀索引用（TEXT/BLOB 列必须填），可留空。")}</div>
    <div id="ix-cols">
      ${columns.map((c) => `
      <div class="ix-col">
        <label class="ct-chk"><input type="checkbox" data-ixcol="${esc(c.name)}"></label>
        <span class="ix-cname">${esc(c.name)}</span>
        <span class="ix-ctype">${esc(c.type)}</span>
        <input type="text" data-ixlen placeholder="${L("长度")}">
      </div>`).join("")}
    </div>`;
  const foot = document.createElement("div");
  foot.innerHTML = `<button class="btn primary">${L("创建")}</button><button class="btn">${L("取消")}</button>`;
  openModal(Lf("创建索引 · {0}", table), body, foot);

  const kindSel = $("#ix-kind", body);
  const nameInp = $("#ix-name", body);
  kindSel.onchange = () => {
    nameInp.disabled = kindSel.value === "primary";
    if (nameInp.disabled) nameInp.value = "";
  };
  const colBoxes = $$("[data-ixcol]", body);
  const lenInputs = $$("[data-ixlen]", body);
  // auto-name when the first column is checked (do not overwrite user input)
  colBoxes.forEach((cb) => (cb.onchange = () => {
    if (nameInp.disabled || nameInp.value.trim()) return;
    const first = colBoxes.filter((x) => x.checked)[0];
    if (first) nameInp.value = "idx_" + first.dataset.ixcol;
  }));

  $(".btn:not(.primary)", foot).onclick = closeModal;
  $(".btn.primary", foot).onclick = async () => {
    const cols = [];
    for (let i = 0; i < colBoxes.length; i++) {
      if (!colBoxes[i].checked) continue;
      const l = lenInputs[i].value.trim();
      if (l && (!/^\d+$/.test(l) || l === "0")) { toast(L("前缀长度必须是正整数"), "err"); return; }
      cols.push({ name: colBoxes[i].dataset.ixcol, length: l || null });
    }
    if (!cols.length) { toast(L("至少勾选一个字段"), "err"); return; }
    try {
      const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/indexes/create`, {
        name: nameInp.value.trim() || null,
        kind: kindSel.value,
        columns: cols,
      });
      toast(res.message, "ok");
      closeModal();
      reloadIdx && reloadIdx();
      refresh && refresh();
    } catch (e) { toast(e.message, "err"); }
  };
}

/** Create-FK modal (pair rows: local column → referenced column; multiple rows form a composite FK) */
function addFkModal(db, table, columns, refresh, reloadFks) {
  const RULE_OPTS = [
    ["", "默认（NO ACTION）"],
    ["cascade", "CASCADE 级联"],
    ["restrict", "RESTRICT 限制"],
    ["no_action", "NO ACTION"],
    ["set_null", "SET NULL"],
    ["set_default", "SET DEFAULT"],
  ];
  const body = document.createElement("div");
  body.style.cssText = "width:min(560px,92vw);";
  body.innerHTML = `
    <div class="inline-form" style="margin-bottom:10px;">
      <div class="field"><label>${L("约束名（留空自动生成）")}</label><input type="text" id="fk-name" style="width:200px"></div>
      <div class="field"><label>${L("引用数据库")}</label><select id="fk-refdb" style="width:130px"></select></div>
      <div class="field"><label>${L("引用表")}</label><select id="fk-reftable" style="width:150px"><option value="">${L("请选择…")}</option></select></div>
    </div>
    <div class="inline-form" style="margin-bottom:10px;align-items:flex-end;">
      <div class="field"><label>ON DELETE</label><select id="fk-ondel" style="width:150px">${RULE_OPTS.map(([v, t]) => `<option value="${v}">${L(t)}</option>`).join("")}</select></div>
      <div class="field"><label>ON UPDATE</label><select id="fk-onupd" style="width:150px">${RULE_OPTS.map(([v, t]) => `<option value="${v}">${L(t)}</option>`).join("")}</select></div>
    </div>
    <div class="gr-gname">${Lf("字段配对（{0} → 引用表）", esc(table))}</div>
    <div id="fk-pairs"></div>
    <button class="btn" id="fk-addpair" style="margin:8px 0 10px;">${L("+ 添加字段对")}</button>
    <div class="hint">${L("每一行是一个字段配对；添加多行即创建复合外键（按行顺序对应）。外键需要 InnoDB 等支持外键的引擎。")}</div>`;
  const foot = document.createElement("div");
  foot.innerHTML = `<button class="btn primary">${L("创建")}</button><button class="btn">${L("取消")}</button>`;
  openModal(Lf("创建外键 · {0}", table), body, foot);

  let refColNames = null; // referenced table columns (loaded after picking a table)
  const pairsEl = $("#fk-pairs", body);

  const pairRow = () => {
    const row = document.createElement("div");
    row.className = "fk-pair";
    row.innerHTML = `
      <select class="fk-my">${columns.map((c) => `<option value="${esc(c.name)}">${esc(c.name)} · ${esc(c.type)}</option>`).join("")}</select>
      <span class="fk-arrow">→</span>
      <select class="fk-ref"></select>
      <button class="btn sm danger-ghost fk-pair-del" title="${L("删除此配对")}">&times;</button>`;
    $(".fk-pair-del", row).onclick = () => {
      if ($$(".fk-pair", pairsEl).length > 1) row.remove();
      else toast(L("至少保留一个字段对"), "err");
    };
    pairsEl.appendChild(row);
    syncRefSelects();
  };
  const syncRefSelects = () => {
    const opts = refColNames
      ? refColNames.map((n) => `<option value="${esc(n)}">${esc(n)}</option>`).join("")
      : "";
    $$(".fk-ref", pairsEl).forEach((sel) => {
      sel.innerHTML = refColNames
        ? opts
        : `<option value="">${L("请先选择引用表…")}</option>`;
      sel.disabled = !refColNames;
    });
  };
  pairRow();
  $("#fk-addpair", body).onclick = pairRow;

  (async () => {
    try {
      const r = await apiGet("/databases");
      $("#fk-refdb", body).innerHTML = r.databases.map((d) => `<option value="${esc(d)}"${d === db ? " selected" : ""}>${esc(d)}</option>`).join("");
      loadRefTables();
    } catch (e) { toast(e.message, "err"); }
  })();

  const loadRefTables = async () => {
    const d = $("#fk-refdb", body).value;
    const sel = $("#fk-reftable", body);
    sel.innerHTML = `<option value="">${L("加载中…")}</option>`;
    try {
      const r = await apiGet(`/databases/${encodeURIComponent(d)}/tables`);
      sel.innerHTML = `<option value="">${L("请选择…")}</option>` + r.tables.map((t) => `<option value="${esc(t.name)}">${esc(t.name)}</option>`).join("");
    } catch (e) { sel.innerHTML = '<option value=""></option>'; toast(e.message, "err"); }
  };
  const loadRefCols = async () => {
    const d = $("#fk-refdb", body).value;
    const t = $("#fk-reftable", body).value;
    if (!t) { refColNames = null; syncRefSelects(); return; }
    try {
      const r = await apiGet(`/databases/${encodeURIComponent(d)}/tables/${encodeURIComponent(t)}/structure`);
      refColNames = r.columns.map((c) => c.name);
    } catch (e) {
      refColNames = null;
      toast(e.message, "err");
    }
    syncRefSelects();
  };
  $("#fk-refdb", body).onchange = () => { refColNames = null; syncRefSelects(); loadRefTables(); };
  $("#fk-reftable", body).onchange = loadRefCols;

  $(".btn:not(.primary)", foot).onclick = closeModal;
  $(".btn.primary", foot).onclick = async () => {
    const myCols = [];
    const refCols = [];
    for (const row of $$(".fk-pair", pairsEl)) {
      const my = $(".fk-my", row).value;
      const ref = $(".fk-ref", row).value;
      if (!my || !ref) { toast(L("每一行都必须选择本表字段和引用字段"), "err"); return; }
      myCols.push(my);
      refCols.push(ref);
    }
    if (new Set(myCols.map((c) => c.toLowerCase())).size !== myCols.length) {
      toast(L("本表字段不能重复"), "err"); return;
    }
    if (new Set(refCols.map((c) => c.toLowerCase())).size !== refCols.length) {
      toast(L("引用字段不能重复"), "err"); return;
    }
    try {
      const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/foreign-keys/create`, {
        name: $("#fk-name", body).value.trim() || null,
        columns: myCols,
        ref_db: $("#fk-refdb", body).value,
        ref_table: $("#fk-reftable", body).value,
        ref_columns: refCols,
        on_delete: $("#fk-ondel", body).value || null,
        on_update: $("#fk-onupd", body).value || null,
      });
      toast(res.message, "ok");
      closeModal();
      reloadFks && reloadFks();
      refresh && refresh();
    } catch (e) { toast(e.message, "err"); }
  };
}

async function renderStructure(db, table, tc) {
  tc.innerHTML = `<div class="panel">${L("加载中...")}</div>`;
  let r;
  try {
    r = await apiGet(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/structure`);
  } catch (e) { tc.innerHTML = `<div class="panel">${esc(L(e.message))}</div>`; return; }
  const rows = r.columns.map((c) => `
    <tr>
      <td><b>${esc(c.name)}</b>${c.key === "PRI" ? " 🔑" : ""}</td>
      <td>${esc(c.type)}</td>
      <td>${c.nullable ? L("是") : L("否")}</td>
      <td>${esc(c.key || "-")}</td>
      <td>${esc(c.extra || "-")}</td>
      <td>${c.default === null ? '<span class="muted">NULL</span>' : esc(c.default)}</td>
      <td class="row-actions">
        <button class="btn sm" data-edit-col="${esc(c.name)}">${L("修改")}</button>
        <button class="btn sm" data-up-col="${esc(c.name)}">${L("上移")}</button>
        <button class="btn sm" data-down-col="${esc(c.name)}">${L("下移")}</button>
        <button class="btn sm danger-ghost" data-drop-col="${esc(c.name)}">${L("删除")}</button>
      </td>
    </tr>`).join("");
  tc.innerHTML = `
    <div class="panel">
      <h2>${L("字段结构")}${r.primaryKey.length ? ` <span class="muted">${Lf("主键：{0}", esc(r.primaryKey.join(", ")))}</span>` : ` <span class="muted">${L("（无主键）")}</span>`}</h2>
      <div class="grid-wrap"><table class="grid">
        <thead><tr><th>${L("字段")}</th><th>${L("类型")}</th><th>${L("可空")}</th><th>${L("键")}</th><th>${L("额外")}</th><th>${L("默认值")}</th><th>${L("操作")}</th></tr></thead>
        <tbody>${rows}</tbody></table></div>
    </div>
    <div class="panel">
      <h3>${L("新增字段")}</h3>
      <div class="inline-form">
        <div class="field"><label>${L("字段名")}</label><input type="text" id="ac-name" style="width:150px"></div>
        <div class="field"><label>${L("类型")}</label><select id="ac-type" style="width:120px">${typeOptions("VARCHAR")}</select></div>
        <div class="field"><label>${L("长度/值")}</label><input type="text" id="ac-len" style="width:140px"></div>
        <div class="field"><label>${L("可空")}</label><label class="ct-chk"><input type="checkbox" id="ac-null"></label></div>
        <div class="field"><label>${L("无符号")}</label><label class="ct-chk"><input type="checkbox" id="ac-unsigned"></label></div>
        <div class="field"><label>${L("自增")}</label><label class="ct-chk"><input type="checkbox" id="ac-ai"></label></div>
        <div class="field"><label>${L("默认值")}</label><input type="text" id="ac-default" style="width:150px" placeholder="${L("留空=无；可填 NULL")}"></div>
        <button class="btn primary" id="ac-submit">${L("添加字段")}</button>
      </div>
    </div>
    <div class="panel">
      <h2>${L("索引")} <span class="spacer"></span><button class="btn" id="idx-add">${L("创建索引")}</button></h2>
      <div class="grid-wrap"><table class="grid">
        <thead><tr><th>${L("名称")}</th><th>${L("类型")}</th><th>${L("方法")}</th><th>${L("字段")}</th><th>${L("操作")}</th></tr></thead>
        <tbody id="idx-body"><tr><td colspan="5" class="muted">${L("加载中...")}</td></tr></tbody>
      </table></div>
    </div>
    <div class="panel">
      <h2>${L("外键")} <span class="spacer"></span><button class="btn" id="fk-add">${L("创建外键")}</button></h2>
      <div class="grid-wrap"><table class="grid">
        <thead><tr><th>${L("名称")}</th><th>${L("本表字段")}</th><th>${L("引用目标")}</th><th>ON DELETE</th><th>ON UPDATE</th><th>${L("操作")}</th></tr></thead>
        <tbody id="fk-body"><tr><td colspan="6" class="muted">${L("加载中...")}</td></tr></tbody>
      </table></div>
    </div>`;
  const refresh = () => renderStructure(db, table, tc);
  const moveCol = async (col, position) => {
    try {
      const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/columns/${encodeURIComponent(col)}/move`, { position });
      toast(res.message, "ok");
      refresh();
    } catch (e) { toast(e.message, "err"); }
  };
  $$("[data-up-col]", tc).forEach((b) => (b.onclick = () => {
    const col = b.dataset.upCol;
    const i = r.columns.findIndex((c) => c.name === col);
    if (i <= 0) { toast(L("已是第一列"), "err"); return; }
    // move up one slot: place AFTER the column two slots above; FIRST when already second
    moveCol(col, i === 1 ? "first" : r.columns[i - 2].name);
  }));
  $$("[data-down-col]", tc).forEach((b) => (b.onclick = () => {
    const col = b.dataset.downCol;
    const i = r.columns.findIndex((c) => c.name === col);
    if (i < 0 || i >= r.columns.length - 1) { toast(L("已是最后一列"), "err"); return; }
    moveCol(col, r.columns[i + 1].name);
  }));
  $$("[data-edit-col]", tc).forEach((b) => (b.onclick = () => {
    const col = b.dataset.editCol;
    const meta = r.columns.find((c) => c.name === col);
    if (meta) modifyColumnModal(db, table, meta, r.primaryKey, refresh);
  }));
  $$("[data-drop-col]", tc).forEach((b) => (b.onclick = async () => {
    const col = b.dataset.dropCol;
    if (!(await confirmModal(L("删除字段"), Lf("确定要删除字段 {0} 吗？该列数据将丢失，不可恢复。", col), true))) return;
    try {
      const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/columns/${encodeURIComponent(col)}/drop`, {});
      toast(res.message, "ok");
      refresh();
    } catch (e) { toast(e.message, "err"); }
  }));
  const syncAcLen = () => { $("#ac-len").placeholder = lenPlaceholder($("#ac-type").value); };
  $("#ac-type").onchange = syncAcLen;
  syncAcLen();
  $("#ac-submit").onclick = async () => {
    const type = $("#ac-type").value;
    const len = $("#ac-len").value.trim();
    const unsigned = $("#ac-unsigned").checked;
    const ai = $("#ac-ai").checked;
    if (T_NEED_LEN.has(type) && !len) { toast(Lf("{0} 类型必须填写长度", type), "err"); return; }
    if (T_VAL_LIST.has(type) && !len) { toast(Lf("{0} 需要取值列表，如 'a','b'", type), "err"); return; }
    if (unsigned && !T_UNSIGNED.has(type)) { toast(Lf("{0} 不支持 UNSIGNED", type), "err"); return; }
    if (ai && !T_AI.has(type)) { toast(Lf("{0} 不支持 AUTO_INCREMENT", type), "err"); return; }
    if (ai && $("#ac-null").checked) { toast(L("自增字段必须为 NOT NULL"), "err"); return; }
    if (ai && $("#ac-default").value.trim()) { toast("自增字段不能设置默认值", "err"); return; }
    try {
      const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/columns/add`, {
        name: $("#ac-name").value.trim(),
        col_type: type,
        length: len,
        unsigned,
        nullable: $("#ac-null").checked,
        auto_increment: ai,
        default: $("#ac-default").value.trim() || null,
      });
      toast(res.message, "ok");
      refresh();
    } catch (e) { toast(e.message, "err"); }
  };

  // index panel
  const loadIndexes = async () => {
    const ib = $("#idx-body", tc);
    try {
      const ix = await apiGet(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/indexes`);
      ib.innerHTML = ix.indexes.length ? ix.indexes.map((i) => `
        <tr>
          <td><b>${esc(i.name)}</b>${i.isPrimary ? " 🔑" : ""}</td>
          <td>${i.isPrimary ? L("主键") : i.unique ? L("唯一") : L("普通")}</td>
          <td>${esc(i.indexType || "-")}</td>
          <td>${i.columns.map((c) => esc(c.name + (c.subPart ? `(${c.subPart})` : ""))).join(", ")}</td>
          <td class="row-actions"><button class="btn sm danger-ghost" data-drop-idx="${esc(i.name)}">${L("删除")}</button></td>
        </tr>`).join("") : `<tr><td colspan="5" class="muted">${L("暂无索引")}</td></tr>`;
      $$("[data-drop-idx]", ib).forEach((b) => (b.onclick = async () => {
        const n = b.dataset.dropIdx;
        if (!(await confirmModal(L("删除索引"), Lf("确定要删除索引 {0} 吗？", n), true))) return;
        try {
          const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/indexes/${encodeURIComponent(n)}/drop`);
          toast(res.message, "ok");
          loadIndexes();
          refresh();
        } catch (e) { toast(e.message, "err"); }
      }));
    } catch (e) {
      ib.innerHTML = `<tr><td colspan="5">${esc(L(e.message))}</td></tr>`;
    }
  };
  loadIndexes();
  $("#idx-add", tc).onclick = () => addIndexModal(db, table, r.columns, refresh, loadIndexes);

  // foreign key panel
  const loadFks = async () => {
    const fb = $("#fk-body", tc);
    try {
      const r2 = await apiGet(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/foreign-keys`);
      fb.innerHTML = r2.foreignKeys.length ? r2.foreignKeys.map((k) => `
        <tr>
          <td><b>${esc(k.name)}</b></td>
          <td>${k.columns.map(esc).join(", ")}</td>
          <td>${esc(k.refDb)}.${esc(k.refTable)} (${k.refColumns.map(esc).join(", ")})</td>
          <td>${esc(k.onDelete || "-")}</td>
          <td>${esc(k.onUpdate || "-")}</td>
          <td class="row-actions"><button class="btn sm danger-ghost" data-drop-fk="${esc(k.name)}">${L("删除")}</button></td>
        </tr>`).join("") : `<tr><td colspan="6" class="muted">${L("暂无外键")}</td></tr>`;
      $$("[data-drop-fk]", fb).forEach((b) => (b.onclick = async () => {
        const n = b.dataset.dropFk;
        if (!(await confirmModal(L("删除外键"), Lf("确定要删除外键 {0} 吗？", n), true))) return;
        try {
          const res = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/foreign-keys/${encodeURIComponent(n)}/drop`);
          toast(res.message, "ok");
          loadFks();
          refresh();
        } catch (e) { toast(e.message, "err"); }
      }));
    } catch (e) {
      fb.innerHTML = `<tr><td colspan="6">${esc(L(e.message))}</td></tr>`;
    }
  };
  loadFks();
  $("#fk-add", tc).onclick = () => addFkModal(db, table, r.columns, refresh, loadFks);
}

/* ---------- per-table SQL tab ---------- */
function renderSqlTab(db, table, tc) {
  renderConsoleInto(tc, db, `SELECT * FROM \`${db}\`.\`${table}\` LIMIT 25;`);
}

/* ---------- table operations ---------- */
function renderOperations(db, table, tc) {
  tc.innerHTML = `
    <div class="panel"><h2>${Lf("表操作 · {0}", esc(table))}</h2>
      <p class="muted">${L("常用维护操作。")}</p>
      <div class="toolbar" style="flex-direction:column;align-items:flex-start;gap:10px">
        <div>
          <button class="btn" id="op-rename">${L("表改名")}</button>
          <button class="btn" id="op-sql">${L("用 SQL 修改更多")}</button>
          <button class="btn" id="op-export">${L("导出此表 (sql.zip)")}</button>
          <button class="btn danger-ghost" id="op-truncate">${L("清空数据（TRUNCATE）")}</button>
          <button class="btn danger-ghost" id="op-drop">${L("删除表（DROP）")}</button>
        </div>
      </div>
    </div>`;
  $("#op-rename").onclick = () => renameTableFlow(db, table, () => showTable(db, S.table, "operations"));
  $("#op-sql").onclick = () => showTable(db, table, "sql");
  $("#op-export").onclick = async () => {
    try {
      await downloadExport(db, [table], `${db}_${table}`);
    } catch (e) { toast(e.message, "err"); }
  };
  $("#op-truncate").onclick = async () => {
    if (!(await confirmModal(L("清空表"), Lf("确定要清空表 {0} 的全部数据吗？此操作不可恢复。", table), true))) return;
    try {
      const r = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/truncate`);
      toast(r.message, "ok");
    } catch (e) { toast(e.message, "err"); }
  };
  $("#op-drop").onclick = async () => {
    if (!(await confirmModal(L("删除表"), Lf("确定要删除表 {0} 吗？表结构与数据都会被删除，不可恢复。", table), true))) return;
    const name = await promptModal(L("确认删除"), Lf("请输入表名 {0} 以确认", table), "");
    if (name !== table) { toast(L("输入不匹配，已取消"), "err"); return; }
    try {
      const r = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/drop`);
      toast(r.message, "ok");
      await loadSidebar();
      showDb(db);
    } catch (e) { toast(e.message, "err"); }
  };
}
async function renameTableFlow(db, table, refresh) {
  const to = await promptModal(L("表改名"), Lf("将表 {0} 重命名为", table), table);
  if (!to || to === table) return;
  try {
    const r = await apiPost(`/databases/${encodeURIComponent(db)}/tables/${encodeURIComponent(table)}/rename`, { to });
    toast(r.message, "ok");
    await loadSidebar();
    if (S.table === table) S.table = to;
    refresh ? refresh() : showDb(db);
  } catch (e) { toast(e.message, "err"); }
}

/* ---------- SQL console ---------- */
function renderConsoleInto(tc, db, prefill) {
  tc.innerHTML = `
    <div class="panel">
      <div class="toolbar">
        <div class="field"><label>${L("目标数据库（执行前先 USE）")}</label>
          <select id="sc-db"><option value="">${L("（不切换）")}</option>
            ${S.dbs.map((d) => `<option value="${esc(d)}" ${d === db ? "selected" : ""}>${esc(d)}</option>`).join("")}
          </select></div>
        <span class="spacer"></span>
        <label class="muted"><input type="checkbox" id="sc-cont"> ${L("出错继续")}</label>
      </div>
      <textarea class="sql-input" id="sc-sql" placeholder="${L("在此输入 SQL，支持多语句，Ctrl+Enter 执行")}">${prefill ? esc(prefill) : ""}</textarea>
      <div class="toolbar" style="margin-top:10px">
        <button class="btn primary" id="sc-run">${L("执行 (Ctrl+Enter)")}</button>
        <button class="btn" id="sc-clear">${L("清空")}</button>
      </div>
      <div id="sc-results" class="sql-result"></div>
    </div>`;
  const run = async () => {
    const sql = $("#sc-sql").value.trim();
    if (!sql) { toast(L("请输入 SQL")); return; }
    $("#sc-results").innerHTML = `<div class="muted">${L("执行中...")}</div>`;
    try {
      const r = await apiPost("/sql", {
        sql,
        db: $("#sc-db").value || null,
        continue_on_error: $("#sc-cont").checked,
      });
      renderConsoleResults($("#sc-results"), r);
    } catch (e) {
      $("#sc-results").innerHTML = `<div class="sql-err">${esc(L(e.message))}</div>`;
    }
  };
  $("#sc-run").onclick = run;
  $("#sc-clear").onclick = () => { $("#sc-sql").value = ""; $("#sc-results").innerHTML = ""; };
  $("#sc-sql").addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") { e.preventDefault(); run(); }
  });
}
function renderConsoleResults(el, r) {
  const parts = [];
  if (r.error) {
    parts.push(`<div class="sql-err">${Lf("第 {0} 条语句执行失败：{1}", r.error.index + 1, esc(L(r.error.message)))}
      <code class="sql-snippet">${esc(r.error.statement)}</code></div>`);
  }
  for (const res of r.results || []) {
    if (res.kind === "rows") {
      const cols = res.columns.map((c) => `<th>${esc(c)}</th>`).join("");
      const rows = res.rows.map((row) => "<tr>" + row.map((v) => (v === null ? '<td class="null">NULL</td>' : `<td>${esc(v)}</td>`)).join("") + "</tr>").join("");
      parts.push(`<div class="stmt"><div class="sql-meta">
          <span class="sql-ok">${Lf("返回 {0} 行", res.rowCount)}${res.truncated ? L("（已截断，仅显示前 1000 行）") : ""}</span>
          <span>${res.ms} ms</span></div>
        ${res.columns.length ? `<div class="grid-wrap"><table class="grid"><thead><tr>${cols}</tr></thead><tbody>${rows}</tbody></table></div>` : `<span class="muted">${L("（无列）")}</span>`}
        <code class="sql-snippet">${esc(res.sql)}</code></div>`);
    } else {
      parts.push(`<div class="stmt"><div class="sql-meta">
          <span class="sql-ok">${Lf("影响 {0} 行", res.affectedRows)}</span>
          ${res.lastInsertId ? `<span>${Lf("自增 ID：{0}", res.lastInsertId)}</span>` : ""}
          <span>${res.ms} ms</span></div>
        <code class="sql-snippet">${esc(res.sql)}</code></div>`);
    }
  }
  parts.push(`<div class="muted">${Lf("共解析 {0} 条语句，成功 {1} 条", r.statements, r.executed)}${r.error ? L("（因出错停止）") : ""}</div>`);
  el.innerHTML = parts.join("");
}
async function showSql(db, prefill) {
  setNav("sql");
  crumbs(L("SQL 控制台"));
  const c = $("#content");
  c.innerHTML = '<div id="console-holder"></div>';
  renderConsoleInto($("#console-holder"), db || null, prefill || "");
}

/* ---------- user management ---------- */
/** Privilege groups (phpMyAdmin-style) */
const PRIV_GROUPS = [
  ["数据", ["SELECT", "INSERT", "UPDATE", "DELETE", "FILE"]],
  ["结构", ["CREATE", "ALTER", "DROP", "INDEX", "CREATE VIEW", "SHOW VIEW", "CREATE TEMPORARY TABLES", "LOCK TABLES", "REFERENCES", "TRIGGER", "EVENT", "CREATE ROUTINE", "ALTER ROUTINE", "EXECUTE"]],
  ["管理", ["GRANT OPTION", "SHOW DATABASES", "CREATE USER", "RELOAD", "PROCESS", "REPLICATION CLIENT", "REPLICATION SLAVE"]],
];
// identifiers used in locally-built GRANT statements must be plain word characters
const IDENT_RE = /^[A-Za-z0-9_$]{1,64}$/;

/** Visual grants modal: current SHOW GRANTS + scope-based GRANT / REVOKE */
function grantsModal(user, host) {
  const body = document.createElement("div");
  body.style.cssText = "width:min(620px,94vw);";
  body.innerHTML = `
    <div class="gr-cur"><label>${L("当前授权（SHOW GRANTS）")}</label><div id="gr-cur-list"></div></div>
    <div class="inline-form" style="margin:12px 0 14px;align-items:flex-end;">
      <div class="field"><label>${L("作用域")}</label><select id="gr-scope" style="width:130px">
        <option value="global">${L("全局 *.*")}</option>
        <option value="db">${L("指定数据库")}</option>
        <option value="table">${L("指定表")}</option></select></div>
      <div class="field"><label>${L("数据库")}</label><select id="gr-db" style="width:160px" disabled></select></div>
      <div class="field"><label>${L("表名")}</label><input type="text" id="gr-table" style="width:140px" disabled placeholder="table_name"></div>
    </div>
    ${PRIV_GROUPS.map(([g, ps]) => `
      <div class="gr-group"><div class="gr-gname">${L(g)}</div>
        <div class="gr-privs">${ps.map((p) => `<label class="gr-chk"><input type="checkbox" value="${p}">${p}</label>`).join("")}</div>
      </div>`).join("")}
    <div class="gr-group"><div class="gr-gname">${L("快捷")}</div>
      <div class="gr-privs"><label class="gr-chk"><input type="checkbox" value="ALL PRIVILEGES">${L("ALL PRIVILEGES（全部权限）")}</label></div>
    </div>
    <div class="hint">${L("勾选权限后点击“授予”或“收回”，将对该用户在所选作用域执行 GRANT / REVOKE。需要当前账号具有相应权限（GRANT OPTION）。")}</div>`;
  const foot = document.createElement("div");
  foot.innerHTML = `<button class="btn primary" id="gr-do-grant">${L("授予勾选权限")}</button><button class="btn danger-ghost" id="gr-do-revoke">${L("收回勾选权限")}</button><button class="btn" id="gr-close">${L("关闭")}</button>`;
  openModal(Lf("授权 · {0}@{1}", user, host), body, foot);

  const loadCur = async () => {
    const el = $("#gr-cur-list", body);
    try {
      const r = await apiGet(`/users/grants?user=${encodeURIComponent(user)}&host=${encodeURIComponent(host)}`);
      el.innerHTML = r.grants.map((g) => `<div class="gr-cmd">${esc(g)}</div>`).join("") || `<span class="muted">${L("（无）")}</span>`;
    } catch (e) { el.innerHTML = `<span class="muted">${esc(L(e.message))}</span>`; }
  };
  loadCur();

  // scope switching + lazy db list
  const scopeSel = $("#gr-scope", body), dbSel = $("#gr-db", body), tblInp = $("#gr-table", body);
  let dbsLoaded = false;
  const loadDbs = async () => {
    if (dbsLoaded) return;
    dbsLoaded = true;
    try {
      const r = await apiGet("/databases");
      dbSel.innerHTML = r.databases.map((d) => `<option>${esc(d)}</option>`).join("");
    } catch (_) { dbSel.innerHTML = ""; }
  };
  const syncScope = () => {
    dbSel.disabled = scopeSel.value === "global";
    tblInp.disabled = scopeSel.value !== "table";
    if (scopeSel.value !== "global") loadDbs();
  };
  scopeSel.onchange = syncScope;

  const apply = async (field) => {
    const checked = $$(".gr-chk input:checked", body).map((x) => x.value);
    if (!checked.length) { toast(L("请先勾选权限"), "err"); return; }
    try {
      const res = await apiPost("/users/grants/apply", {
        user, host,
        scope: scopeSel.value,
        database: dbSel.disabled ? null : dbSel.value,
        table: tblInp.disabled ? null : tblInp.value.trim(),
        grant: field === "grant" ? checked : [],
        revoke: field === "revoke" ? checked : [],
      });
      toast(res.message, "ok");
      loadCur();
    } catch (e) { toast(e.message, "err"); }
  };
  $("#gr-do-grant", foot).onclick = () => apply("grant");
  $("#gr-do-revoke", foot).onclick = () => apply("revoke");
  $("#gr-close", foot).onclick = closeModal;
}

async function showUsers() {
  setNav("users");
  crumbs(L("用户管理"));
  const c = $("#content");
  c.innerHTML = `<div class="panel">${L("加载中...")}</div>`;
  let users;
  try {
    users = (await apiGet("/users")).users;
  } catch (e) {
    c.innerHTML = `
      <div class="panel"><h2>${L("用户管理")}</h2>
        <div class="notice warn">${esc(L(e.message))}</div>
        <p class="muted">${L("可以尝试通过 SQL 控制台执行 CREATE USER / GRANT 等语句（若服务器允许）。")}</p>
        <div class="toolbar"><button class="btn" id="us-sql">${L("打开 SQL 控制台")}</button></div></div>`;
    $("#us-sql").onclick = () => showSql();
    return;
  }
  const rows = users.map((u, i) => `
    <tr>
      <td><b>${esc(u.user)}</b></td>
      <td>${esc(u.host)}</td>
      <td>${esc(u.plugin || "-")}</td>
      <td class="row-actions">
        <button class="btn sm" data-pass="${i}">${L("改密码")}</button>
        <button class="btn sm" data-grants="${i}">${L("授权")}</button>
        <button class="btn sm" data-rename="${i}">${L("改名")}</button>
        <button class="btn sm danger-ghost" data-drop="${i}">${L("删除")}</button>
      </td>
    </tr>`).join("");
  c.innerHTML = `
    <div class="panel">
      <h2>${L("数据库用户")}
        <span class="spacer"></span>
        <button class="btn primary" id="us-create">${L("创建用户")}</button>
        <button class="btn" id="us-flush">${L("重载权限 (FLUSH PRIVILEGES)")}</button>
      </h2>
      <div class="grid-wrap"><table class="grid">
        <thead><tr><th>${L("用户名")}</th><th>${L("主机")}</th><th>${L("认证插件")}</th><th>${L("操作")}</th></tr></thead>
        <tbody>${rows}</tbody></table></div>
    </div>`;
  const refresh = () => showUsers();
  $("#us-create").onclick = async () => {
    const body = document.createElement("div");
    body.innerHTML = `<div class="form-grid">
      <div class="field"><label>${L("用户名")}</label><input type="text" id="nu-user"></div>
      <div class="field"><label>${L("主机（留空 = localhost）")}</label><input type="text" id="nu-host" placeholder="localhost"></div>
      <div class="field"><label>${L("初始密码")}</label><input type="password" id="nu-pass"></div>
      <div class="field"><label>${L("授权（可选）")}</label><select id="nu-grant">
        <option value="">${L("不授权")}</option>
        <option value="all">${L("ALL PRIVILEGES ON *.*（超级用户）")}</option>
        <option value="alldb">${L("ALL PRIVILEGES ON 新库.*（稍后自行 GRANT）")}</option>
      </select></div>
      <div class="field"><label>${L("数据库名（选“新库”时填写）")}</label><input type="text" id="nu-db"></div>
    </div>`;
    const foot = document.createElement("div");
    foot.innerHTML = `<button class="btn primary">${L("创建")}</button><button class="btn">${L("取消")}</button>`;
    openModal(L("创建用户"), body, foot);
    $(".btn.primary", foot).onclick = async () => {
      const user = $("#nu-user", body).value.trim();
      const host = $("#nu-host", body).value.trim() || "localhost";
      const pass = $("#nu-pass", body).value;
      if (!user) { toast(L("用户名不能为空"), "err"); return; }
      // user/host/db go into locally-built backticked GRANT SQL: only allow word chars
      if (!IDENT_RE.test(user) || !IDENT_RE.test(host)) { toast(L("用户名/主机名含非法字符"), "err"); return; }
      const g = $("#nu-grant", body).value;
      const gdb = $("#nu-db", body).value.trim();
      if (gdb && !IDENT_RE.test(gdb)) { toast(L("数据库名含非法字符"), "err"); return; }
      try {
        await apiPost("/users/create", { user, host, password: pass });
        if (g === "all") {
          await apiPost("/sql", { sql: `GRANT ALL PRIVILEGES ON *.* TO \`${user}\`@\`${host}\` WITH GRANT OPTION;` });
        } else if (g === "alldb" && gdb) {
          await apiPost("/sql", { sql: `GRANT ALL PRIVILEGES ON \`${gdb}\`.* TO \`${user}\`@\`${host}\`;` });
        }
        toast(Lf("用户 {0}@{1} 已创建", user, host), "ok");
        closeModal();
        refresh();
      } catch (e) { toast(e.message, "err"); }
    };
    $(".btn:not(.primary)", foot).onclick = closeModal;
  };
  $("#us-flush").onclick = async () => {
    try { const r = await apiPost("/flush-privileges"); toast(r.message, "ok"); } catch (e) { toast(e.message, "err"); }
  };
  $$("[data-pass]", c).forEach((b) => (b.onclick = async () => {
    const u = users[parseInt(b.dataset.pass)];
    const pass = await promptModal(L("修改密码"), Lf("为用户 {0}@{1} 设置新密码", u.user, u.host), "", L("使用 ALTER USER ... IDENTIFIED BY"));
    if (!pass) return;
    try {
      const r = await apiPost("/users/password", { user: u.user, host: u.host, password: pass });
      toast(r.message, "ok");
    } catch (e) { toast(e.message, "err"); }
  }));
  $$("[data-grants]", c).forEach((b) => (b.onclick = () => {
    const u = users[parseInt(b.dataset.grants)];
    grantsModal(u.user, u.host);
  }));
  $$("[data-rename]", c).forEach((b) => (b.onclick = async () => {
    const u = users[parseInt(b.dataset.rename)];
    const to = await promptModal(L("用户改名"), Lf("将用户 {0}@{1} 重命名为", u.user, u.host), u.user);
    if (!to || to === u.user) return;
    try {
      const r = await apiPost("/users/rename", { user: u.user, host: u.host, to });
      toast(r.message, "ok");
      refresh();
    } catch (e) { toast(e.message, "err"); }
  }));
  $$("[data-drop]", c).forEach((b) => (b.onclick = async () => {
    const u = users[parseInt(b.dataset.drop)];
    if (!(await confirmModal(L("删除用户"), Lf("确定要删除用户 {0}@{1} 吗？", u.user, u.host), true))) return;
    try {
      const r = await apiPost("/users/drop", { user: u.user, host: u.host });
      toast(r.message, "ok");
      refresh();
    } catch (e) { toast(e.message, "err"); }
  }));
}

/* ---------- import ---------- */
async function showImport(targetDb) {
  setNav("import");
  crumbs(L("导入 SQL"));
  const c = $("#content");
  c.innerHTML = `
    <div class="panel">
      <h2>${L("导入 SQL 文件")}</h2>
      <p class="muted">${L("支持 .sql 文件或包含 .sql 的 .zip（如本工具导出的 xxx.sql.zip）。导入将逐条执行所有语句，遇错停止。")}</p>
      <div class="form-grid">
        <div class="field"><label>${L("目标数据库（可选；将在导入前 CREATE DATABASE IF NOT EXISTS + USE）")}</label>
          <select id="im-db"><option value="">${L("（不指定，使用文件内的 USE / 库名限定）")}</option>
            ${S.dbs.map((d) => `<option value="${esc(d)}" ${d === targetDb ? "selected" : ""}>${esc(d)}</option>`).join("")}
          </select></div>
        <div class="field"><label>${L("选择文件（.sql / .zip）")}</label><input type="file" id="im-file" accept=".sql,.zip,.txt"></div>
      </div>
      <div class="toolbar" style="margin-top:14px">
        <button class="btn primary" id="im-go" disabled>${L("开始导入")}</button>
        <span class="muted" id="im-note">${L("未选择文件")}</span>
      </div>
      <div id="im-result"></div>
    </div>`;
  $("#im-file").onchange = (e) => {
    $("#im-go").disabled = !e.target.files.length;
    $("#im-note").textContent = e.target.files.length ? e.target.files[0].name : L("未选择文件");
  };
  $("#im-go").onclick = async () => {
    const f = $("#im-file").files[0];
    if (!f) return;
    const fd = new FormData();
    fd.append("file", f);
    fd.append("target_db", $("#im-db").value);
    $("#im-go").disabled = true;
    $("#im-go").textContent = L("导入中...");
    $("#im-result").innerHTML = `<div class="muted">${L("正在上传并执行，请勿关闭页面...")}</div>`;
    try {
      const res = await fetch("/api/import", {
        method: "POST",
        headers: { "x-auth-token": S.token },
        body: fd,
      });
      const data = await res.json();
      if (!res.ok) throw new Error(L(data.error) || L("导入失败"));
      renderConsoleResults($("#im-result"), data.result);
      const ok = !data.result.error;
      toast(ok ? Lf("导入完成：成功执行 {0} 条语句", data.result.executed) : L("导入中断（部分语句已执行）"), ok ? "ok" : "err");
      await loadSidebar();
    } catch (e) {
      $("#im-result").innerHTML = `<div class="sql-err">${esc(L(e.message))}</div>`;
      toast(e.message, "err");
    } finally {
      $("#im-go").disabled = false;
      $("#im-go").textContent = L("开始导入");
    }
  };
}

/* ---------- export ---------- */
async function downloadExport(db, tables, nameHint, mode) {
  const p = new URLSearchParams({ db });
  if (tables && tables.length) p.set("tables", tables.join(","));
  if (mode) p.set("mode", mode);
  const res = await fetch("/api/export?" + p, { headers: { "x-auth-token": S.token } });
  if (!res.ok) {
    let m = L("导出失败");
    try { m = L((await res.json()).error) || m; } catch (_) {}
    throw new Error(m);
  }
  const blob = await res.blob();
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = (nameHint || db) + ".sql.zip";
  document.body.appendChild(a);
  a.click();
  setTimeout(() => { URL.revokeObjectURL(a.href); a.remove(); }, 2000);
  toast(L("导出完成，已开始下载"), "ok");
}
async function showExport(db) {
  setNav("export");
  crumbs(L("导出 SQL"));
  const c = $("#content");
  c.innerHTML = `
    <div class="panel">
      <h2>${L("导出数据库为 sql.zip")}</h2>
      <div class="inline-form">
        <div class="field"><label>${L("数据库")}</label>
          <select id="ex-db">${S.dbs.map((d) => `<option value="${esc(d)}" ${d === db ? "selected" : ""}>${esc(d)}</option>`).join("")}</select></div>
        <div class="field"><label>${L("导出内容")}</label>
          <select id="ex-mode" style="width:150px">
            <option value="full">${L("结构和数据")}</option>
            <option value="structure">${L("仅结构")}</option>
            <option value="data">${L("仅数据")}</option>
          </select></div>
        <button class="btn primary" id="ex-go">${L("导出并下载")}</button>
      </div>
      <div style="margin-top:14px">
        <h3>${L("选择表（不选 = 全部）")}</h3>
        <div id="ex-tables" class="muted">${L("请选择数据库")}</div>
      </div>
    </div>`;
  const loadTables = async () => {
    const d = $("#ex-db").value;
    const holder = $("#ex-tables");
    holder.innerHTML = L("加载中...");
    try {
      const r = await apiGet(`/databases/${encodeURIComponent(d)}/tables`);
      if (!r.tables.length) { holder.innerHTML = `<span class="muted">${L("（该库无表）")}</span>`; return; }
      holder.innerHTML = r.tables.map((t) => `
        <label style="display:inline-flex;align-items:center;gap:5px;margin:3px 10px 3px 0;padding:4px 10px;background:#f1f5f9;border-radius:6px;font-size:12.5px;cursor:pointer">
          <input type="checkbox" value="${esc(t.name)}" checked>${esc(t.name)}</label>`).join("");
    } catch (e) { holder.innerHTML = esc(L(e.message)); }
  };
  $("#ex-db").onchange = loadTables;
  loadTables();
  $("#ex-go").onclick = async () => {
    const d = $("#ex-db").value;
    const sel = $$("#ex-tables input[type=checkbox]:checked").map((i) => i.value);
    const all = $$("#ex-tables input[type=checkbox]").length;
    try {
      await downloadExport(d, sel.length && sel.length < all ? sel : null, d, $("#ex-mode").value);
    } catch (e) { toast(e.message, "err"); }
  };
}

/* ---------- top navigation ---------- */
function bindTopNav() {
  $$("[data-nav]").forEach((b) => {
    b.addEventListener("click", () => {
      const v = b.dataset.nav;
      if (v === "home") showHome();
      else if (v === "sql") showSql(S.db || null);
      else if (v === "users") showUsers();
      else if (v === "import") showImport();
      else if (v === "export") showExport();
    });
  });
}

/* ---------- bootstrap ---------- */
document.addEventListener("DOMContentLoaded", () => {
  document.documentElement.lang = LANG;
  bindLogin();
  bindTopNav();
  applyStaticLang();
  if (S.token) initApp();
  else showLogin();
});
