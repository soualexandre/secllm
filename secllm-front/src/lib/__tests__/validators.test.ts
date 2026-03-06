import { loginSchema, registerSchema, createClientSchema } from "../validators";

describe("validators", () => {
  describe("loginSchema", () => {
    it("accepts valid email and password", () => {
      const result = loginSchema.safeParse({ email: "a@b.com", password: "secret" });
      expect(result.success).toBe(true);
    });
    it("rejects invalid email", () => {
      const result = loginSchema.safeParse({ email: "invalid", password: "secret" });
      expect(result.success).toBe(false);
    });
  });

  describe("registerSchema", () => {
    it("accepts valid input", () => {
      const result = registerSchema.safeParse({
        email: "a@b.com",
        password: "password123",
        name: "Test",
      });
      expect(result.success).toBe(true);
    });
    it("rejects short password", () => {
      const result = registerSchema.safeParse({
        email: "a@b.com",
        password: "short",
      });
      expect(result.success).toBe(false);
    });
  });

  describe("createClientSchema", () => {
    it("accepts valid client_id", () => {
      const result = createClientSchema.safeParse({ client_id: "my-app" });
      expect(result.success).toBe(true);
    });
    it("rejects empty client_id", () => {
      const result = createClientSchema.safeParse({ client_id: "" });
      expect(result.success).toBe(false);
    });
  });
});
