/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["templates/sources/**/*.html", "templates/**/*.yml"],
  safelist: [
    {
      pattern: /alert-*/,
    }
  ],
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

