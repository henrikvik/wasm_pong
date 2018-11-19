const path = require("path");
const HtmlPlugin = require("html-webpack-plugin");

module.exports = {
    entry: "./index.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "index.js"
    },
    plugins: [
        new HtmlPlugin({ title: "wasm_pong" })
    ],
    mode: "development"
}