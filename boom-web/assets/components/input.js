window.addEventListener("DOMContentLoaded", () => {
  const boomCheckboxes = Array.from(
    document.querySelectorAll(".boom-input:has(> input[type='checkbox'])"),
  );
  boomCheckboxes.forEach((checkbox) => {
    const changeEvent = new Event("change");

    checkbox.addEventListener("click", () => {
      const checkboxInput = /** @type {HTMLInputElement} **/ (
        checkbox.querySelector("input[type='checkbox']")
      );

      checkboxInput.checked = !checkboxInput.checked;
      checkboxInput.dispatchEvent(changeEvent);
    });
  });
});
