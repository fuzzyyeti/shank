#[derive(ShankInstruction)]
pub enum Instruction {
    #[account(0, name = "creator", sig)]
    #[account(1, name = "thing", mut)]
    #[discriminant(10)]
    CreateThing,
    #[account(name = "original_creator", sig)]
    #[discriminant(10,20,30,40,50,60,70,80)]
    CloseThing,
}
