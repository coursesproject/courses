/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["templates/sources/**/*.html", "templates/**/*.yml"],
  theme: {
    extend: {},
  },
  plugins: [
    require("@tailwindcss/typography"),
    require("daisyui")],
  daisyui: {
    themes: ["light", "dark", "retro", "cupcake"]
  }
}

