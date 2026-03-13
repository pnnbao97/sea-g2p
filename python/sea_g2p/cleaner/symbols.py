import string

vietnamese_set = '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYỹỷỵỳựữửừứủụợỡởờớộỗổồốỏọịỉệễểềếẽẻẹặẵẳằắậẫẩầấảạươũĩđăýúùõôóòíìêéèãâàáÀÁÂÃÈÉÊÌÍÒÓÔÕÙÚÝĂĐĨŨƠƯẠẢẤẦẨẪẬẮẰẴẶẸẺẼẾỀỂỄỆỈỊỌỎỐỒỔỖỘỚỜỞỠỢỤỦỨỪỬỮỰỲỴỶỸ'

opening_brackets_and_punctutations_re = r'([\(\[\{])(\s)'
punctutations_re = r'(\s)([\.,\?\!\:\;\)\]\}])'
punctuations = set([i for i in string.punctuation] + ['“‘”’'])
