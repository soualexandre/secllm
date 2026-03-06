import { test, expect } from "@playwright/test";

test("login page loads", async ({ page }) => {
  await page.goto("/login");
  await expect(page.getByRole("heading", { name: /sign in/i })).toBeVisible();
});

test("register page loads", async ({ page }) => {
  await page.goto("/register");
  await expect(page.getByRole("heading", { name: /create account/i })).toBeVisible();
});
