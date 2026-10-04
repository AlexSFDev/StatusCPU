module statuscpu (
    input  logic clk,
    input  logic reset,

    output logic [31:0] mem_addr,
    output logic [31:0] mem_wdata,
    output logic [3:0]  mem_wstrb,
    output logic        mem_valid,
    input  logic [31:0] mem_rdata,
    input  logic        mem_ready,

    input  logic        irq_timer,
    input  logic        irq_external,

    output logic        uart_valid,
    output logic [7:0]  uart_data,

    output logic        halted
);

localparam [31:0] UART_BASE = 32'h80000000;
localparam [31:0] TIMER_BASE = 32'h80000100;
localparam [31:0] IRQ_BASE = 32'h80000200;

localparam [31:0] FLAG_Z = 32'h00000001;
localparam [31:0] FLAG_N = 32'h00000002;
localparam [31:0] FLAG_C = 32'h00000004;
localparam [31:0] FLAG_V = 32'h00000008;

localparam [31:0] CAUSE_ILLEGAL = 32'd2;
localparam [31:0] CAUSE_BREAK = 32'd3;
localparam [31:0] CAUSE_DIVZERO = 32'd5;
localparam [31:0] CAUSE_TIMER = 32'h80000001;
localparam [31:0] CAUSE_EXTERNAL = 32'h80000002;

logic [31:0] regs [0:15];

logic [31:0] pc;
logic [31:0] flags;

logic [31:0] status;
logic [31:0] cause;
logic [31:0] epc;
logic [31:0] tvec;
logic [31:31] reserved0;
logic [31:0] scratch;
logic [31:0] mode;
logic [31:0] ie;
logic [31:0] ip;

logic [63:0] cycle;
logic [63:0] instret;

logic [31:0] timer;
logic [31:0] timer_compare;
logic [31:0] timer_control;

logic [31:0] instruction;
logic [7:0] opcode;

logic [4:0] rd;
logic [4:0] rs1;
logic [4:0] rs2;

logic [31:0] alu_a;
logic [31:0] alu_b;
logic [31:0] alu_result;

logic sleeping;

integer i;

function automatic [31:0] reg_read(input [3:0] index);
    begin
        if (index == 0)
            reg_read = 32'd0;
        else
            reg_read = regs[index];
    end
endfunction

task automatic reg_write(input [3:0] index, input [31:0] value);
    begin
        if (index != 0)
            regs[index] <= value;
        regs[0] <= 32'd0;
    end
endtask

always_comb begin
    mem_addr = 32'd0;
    mem_wdata = 32'd0;
    mem_wstrb = 4'd0;
    mem_valid = 1'b0;

    uart_valid = 1'b0;
    uart_data = 8'd0;

    if (!reset && !halted && !sleeping) begin
        mem_addr = pc;
        mem_valid = 1'b1;
    end
end

always_ff @(posedge clk) begin
    if (reset) begin
        pc <= 32'h00000000;
        flags <= 32'd0;

        status <= 32'd0;
        cause <= 32'd0;
        epc <= 32'd0;
        tvec <= 32'hFFFF0100;
        scratch <= 32'd0;
        mode <= 32'd2;
        ie <= 32'd0;
        ip <= 32'd0;

        cycle <= 64'd0;
        instret <= 64'd0;

        timer <= 32'd0;
        timer_compare <= 32'hFFFFFFFF;
        timer_control <= 32'd0;

        sleeping <= 1'b0;
        halted <= 1'b0;

        for (i = 0; i < 16; i = i + 1)
            regs[i] <= 32'd0;

        regs[15] <= 32'h7FFFFFFC;
    end else begin
        cycle <= cycle + 1;

        timer <= timer + 1;

        if ((timer_control & 32'd3) == 32'd3 &&
            timer >= timer_compare)
            ip[0] <= 1'b1;

        if (irq_timer)
            ip[0] <= 1'b1;

        if (irq_external)
            ip[1] <= 1'b1;

        if (sleeping && ((status & 1) != 0) && ((ip & ie) != 0))
            sleeping <= 1'b0;

        if (!halted && !sleeping && mem_ready) begin
            instruction <= mem_rdata;
            opcode <= mem_rdata[7:0];

            rd <= mem_rdata[11:8];
            rs1 <= mem_rdata[15:12];
            rs2 <= mem_rdata[19:16];

            case (mem_rdata[7:0])

                8'h00: begin
                    halted <= 1'b1;
                end

                8'h01: begin
                    pc <= pc + 1;
                    instret <= instret + 1;
                end

                8'h02: begin
                    sleeping <= 1'b1;
                    pc <= pc + 1;
                    instret <= instret + 1;
                end

                8'h03: begin
                    pc <= epc;
                    instret <= instret + 1;
                end

                8'h04: begin
                    cause <= CAUSE_BREAK;
                    epc <= pc;
                    pc <= tvec;
                    status[0] <= 1'b0;
                end

                8'h10: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) + mem_rdata[31:16]
                    );
                    pc <= pc + 7;
                    instret <= instret + 1;
                end

                8'h11: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) - mem_rdata[31:16]
                    );
                    pc <= pc + 7;
                    instret <= instret + 1;
                end

                8'h12: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) & mem_rdata[31:16]
                    );
                    pc <= pc + 7;
                    instret <= instret + 1;
                end

                8'h13: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) | mem_rdata[31:16]
                    );
                    pc <= pc + 7;
                    instret <= instret + 1;
                end

                8'h14: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) ^ mem_rdata[31:16]
                    );
                    pc <= pc + 7;
                    instret <= instret + 1;
                end

                8'h15: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) * mem_rdata[31:16]
                    );
                    pc <= pc + 7;
                    instret <= instret + 1;
                end

                8'h20: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) +
                        reg_read(mem_rdata[19:16])
                    );
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'h21: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) -
                        reg_read(mem_rdata[19:16])
                    );
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'h22: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) *
                        reg_read(mem_rdata[19:16])
                    );
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'h23: begin
                    if (reg_read(mem_rdata[19:16]) == 0) begin
                        cause <= CAUSE_DIVZERO;
                        epc <= pc;
                        pc <= tvec;
                        status[0] <= 1'b0;
                    end else begin
                        reg_write(
                            mem_rdata[11:8],
                            $signed(reg_read(mem_rdata[15:12])) /
                            $signed(reg_read(mem_rdata[19:16]))
                        );
                        pc <= pc + 4;
                        instret <= instret + 1;
                    end
                end

                8'h24: begin
                    if (reg_read(mem_rdata[19:16]) == 0) begin
                        cause <= CAUSE_DIVZERO;
                        epc <= pc;
                        pc <= tvec;
                        status[0] <= 1'b0;
                    end else begin
                        reg_write(
                            mem_rdata[11:8],
                            reg_read(mem_rdata[15:12]) /
                            reg_read(mem_rdata[19:16])
                        );
                        pc <= pc + 4;
                        instret <= instret + 1;
                    end
                end

                8'h30: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) &
                        reg_read(mem_rdata[19:16])
                    );
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'h31: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) |
                        reg_read(mem_rdata[19:16])
                    );
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'h32: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) ^
                        reg_read(mem_rdata[19:16])
                    );
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'h33: begin
                    reg_write(
                        mem_rdata[11:8],
                        ~reg_read(mem_rdata[15:12])
                    );
                    pc <= pc + 3;
                    instret <= instret + 1;
                end

                8'h40: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) <<
                        (reg_read(mem_rdata[19:16]) & 31)
                    );
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'h41: begin
                    reg_write(
                        mem_rdata[11:8],
                        reg_read(mem_rdata[15:12]) >>
                        (reg_read(mem_rdata[19:16]) & 31)
                    );
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'h42: begin
                    reg_write(
                        mem_rdata[11:8],
                        $signed(reg_read(mem_rdata[15:12])) >> 
                        (reg_read(mem_rdata[19:16]) & 31)
                    );
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'h60: begin
                    pc <= mem_rdata[31:0];
                    instret <= instret + 1;
                end

                8'h61: begin
                    if (flags & FLAG_Z)
                        pc <= mem_rdata[31:0];
                    else
                        pc <= pc + 5;
                    instret <= instret + 1;
                end

                8'h62: begin
                    if (!(flags & FLAG_Z))
                        pc <= mem_rdata[31:0];
                    else
                        pc <= pc + 5;
                    instret <= instret + 1;
                end

                8'h70: begin
                    regs[15] <= regs[15] - 4;
                    pc <= pc + 3;
                    instret <= instret + 1;
                end

                8'h71: begin
                    regs[15] <= regs[15] + 4;
                    pc <= pc + 3;
                    instret <= instret + 1;
                end

                8'h90: begin
                    case (mem_rdata[23:16])
                        8'h00: reg_write(mem_rdata[11:8], status);
                        8'h01: reg_write(mem_rdata[11:8], cause);
                        8'h02: reg_write(mem_rdata[11:8], epc);
                        8'h03: reg_write(mem_rdata[11:8], tvec);
                        8'h04: reg_write(mem_rdata[11:8], scratch);
                        8'h07: reg_write(mem_rdata[11:8], mode);
                        8'h08: reg_write(mem_rdata[11:8], ie);
                        8'h09: reg_write(mem_rdata[11:8], ip);
                        8'h0A: reg_write(mem_rdata[11:8], 32'h53544350);
                        default: reg_write(mem_rdata[11:8], 0);
                    endcase
                    pc <= pc + 4;
                    instret <= instret + 1;
                end

                8'hA0: begin
                    pc <= epc;
                    status[0] <= 1'b1;
                    mode <= 2;
                    instret <= instret + 1;
                end

                8'hA1: begin
                    pc <= epc;
                    status[0] <= 1'b1;
                    mode <= 1;
                    instret <= instret + 1;
                end

                8'hA2: begin
                    if (mode == 2)
                        mode <= reg_read(mem_rdata[11:8]) & 2;
                    else begin
                        cause <= 32'd6;
                        epc <= pc;
                        pc <= tvec;
                        status[0] <= 1'b0;
                    end
                end

                8'hB0: begin
                    cause <= mem_rdata[31:0];
                    epc <= pc;
                    pc <= tvec;
                    status[0] <= 1'b0;
                    mode <= 2;
                end

                8'hB1: begin
                    pc <= epc;
                    status[0] <= 1'b1;
                    mode <= 2;
                    instret <= instret + 1;
                end

                default: begin
                    cause <= CAUSE_ILLEGAL;
                    epc <= pc;
                    pc <= tvec;
                    status[0] <= 1'b0;
                    mode <= 2;
                end
            endcase

            regs[0] <= 32'd0;
        end
    end
end

endmodule