const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');
const HtmlWebpackPlugin =  require('html-webpack-plugin');
const TsconfigPathsPlugin = require('tsconfig-paths-webpack-plugin');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");

const path = require('path');

const devMode = process.env.NODE_ENV !== "production";

const sharedConfig = {
  mode: devMode ? "development" : "production",
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
    plugins: [new TsconfigPathsPlugin({ configFile: "./tsconfig.json" })]
  },
};

const sharedRules = [
  {
    test: /\.tsx?$/,
    use: 'ts-loader',
    exclude: /node_modules/,
  },
];



const browserConfig = {
  ...sharedConfig,

  entry: "./src/bootstrap_browser.js",
  output: {
    path: path.resolve(__dirname, devMode ? "dist-dev" : "dist-prod"),
    filename: "browser.js",
  },

  devServer: {
    // watchFiles: ['./index.html'],
    // hot: true,
    liveReload: true,
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: './src/browser/index.html'
    }),
    new MonacoWebpackPlugin({
      languages: [],
    }),
    new MiniCssExtractPlugin()
  ],
  module: {
    rules: [
      ...sharedRules,

      // {
			// 	test: /\.css$/,
			// 	use: ['style-loader', 'css-loader']
			// },
      {
        //test: /\.s[ac]ss$/i,
        test: /\.(sa|sc|c)ss$/i,
        use: [
          // Creates `style` nodes from JS strings
          //"style-loader",
          MiniCssExtractPlugin.loader,
          // Translates CSS into CommonJS
          "css-loader",
          // Compiles Sass to CSS
          "sass-loader",
        ],
      },
    ],
  },
};

const workerConfig = {
  ...sharedConfig,

  entry: "./src/bootstrap_worker.js",
  output: {
    path: path.resolve(__dirname, devMode ? "dist-dev" : "dist-prod"),
    filename: "worker.js",
  },

  target: 'webworker',
  experiments: {
    syncWebAssembly: true,
  },

  module: {
    rules: [
      ...sharedRules,
    ]
  },
}

module.exports = [
  browserConfig,
  workerConfig,
]
