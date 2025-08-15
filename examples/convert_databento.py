from hftbacktest.data.utils.databento import convert
symbol = "PAPL"

input_filename = "PAPL_20250723_mbo.dbn.zst"

convert(input_filename, symbol, output_filename="PAPL_20250723_mbo2.npz")