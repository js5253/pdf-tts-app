export default function (eleventyConfig) {
  // Copy the `src/css/` directory to the output folder (e.g., _site/css/)
  eleventyConfig.addPassthroughCopy("src/css");
  eleventyConfig.addPassthroughCopy({"public/*": "/"});

  return {
    dir: {
      input: "src",    // Source files directory
      output: "_site"  // Compiled site directory
    }
  };
};