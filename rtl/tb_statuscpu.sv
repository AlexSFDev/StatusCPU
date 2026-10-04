module tb_statuscpu;

logic clk;
logic reset;

logic [31:0] mem_addr;
logic [31:0] mem_wdata;
logic [3:0] mem_wstrb;
logic mem_valid;
logic [31:0] mem_rdata;
logic mem_ready;

logic irq_timer;
logic irq_external;

logic uart_valid;
logic [7:0] uart_data;
logic halted;

logic [31:0] memory [0:1023];

integer i;

statuscpu cpu (
    .clk(clk),
    .reset(reset),
    .mem_addr(mem_addr),
    .mem_wdata(mem_wdata),
    .mem_wstrb(mem_wstrb),
    .mem_valid(mem_valid),
    .mem_rdata(mem_rdata),
    .mem_ready(mem_ready),
    .irq_timer(irq_timer),
    .irq_external(irq_external),
    .uart_valid(uart_valid),
    .uart_data(uart_data),
    .halted(halted)
);

always #5 clk = ~clk;

always_comb begin
    if (mem_addr < 4096)
        mem_rdata = memory[mem_addr >> 2];
    else
        mem_rdata = 32'd0;

    mem_ready = mem_valid;
end

always_ff @(posedge clk) begin
    if (mem_valid && |mem_wstrb && mem_addr < 4096) begin
        if (mem_wstrb[0])
            memory[mem_addr][7:0] <= mem_wdata[7:0];

        if (mem_wstrb[1])
            memory[mem_addr][15:8] <= mem_wdata[15:8];

        if (mem_wstrb[2])
            memory[mem_addr][23:16] <= mem_wdata[23:16];

        if (mem_wstrb[3])
            memory[mem_addr][31:24] <= mem_wdata[31:24];
    end
end

initial begin
    clk = 0;
    reset = 1;
    irq_timer = 0;
    irq_external = 0;

    for (i = 0; i < 1024; i = i + 1)
        memory[i] = 0;

    #20;

    reset = 0;

    #1000;

    $display("StatusCPU RTL test complete");
    $display("PC: %h", cpu.pc);
    $display("R1: %h", cpu.regs[1]);
    $display("R2: %h", cpu.regs[2]);
    $display("R3: %h", cpu.regs[3]);

    $finish;
end

endmodule