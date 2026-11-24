import ts from "typescript";
import fs from "node:fs";
import assert from "node:assert/strict";
const catalog = JSON.parse(
  fs.readFileSync(
    new URL("../../../locales/en-US.json", import.meta.url),
    "utf8",
  ),
);
for (const file of fs.readdirSync("src").filter((f) => f.endsWith(".tsx"))) {
  const source = ts.createSourceFile(
    file,
    fs.readFileSync(`src/${file}`, "utf8"),
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  function visit(node) {
    if (ts.isJsxText(node)) {
      const text = node.text.trim();
      assert(
        !/[A-Za-z]{2}/.test(text) || /^v\d[\d.]*$/.test(text),
        `Uncatalogued JSX text in ${file}: ${text}`,
      );
    }
    if (
      ts.isCallExpression(node) &&
      node.expression.getText(source) === "t" &&
      ts.isStringLiteral(node.arguments[0])
    )
      assert(
        node.arguments[0].text in catalog,
        `Missing key ${node.arguments[0].text}`,
      );
    ts.forEachChild(node, visit);
  }
  visit(source);
}
console.log("Frontend catalog and JSX text coverage passed.");
