import string

vietnamese_set = '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYỹỷỵỳựữửừứủụợỡởờớộỗổồốỏọịỉệễểềếẽẻẹặẵẳằắậẫẩầấảạươũĩđăýúùõôóòíìêéèãâàáÀÁÂÃÈÉÊÌÍÒÓÔÕÙÚÝĂĐĨŨƠƯẠẢẤẦẨẪẬẮẰẴẶẸẺẼẾỀỂỄỆỈỊỌỎỐỒỔỖỘỚỜỞỠỢỤỦỨỪỬỮỰỲỴỶỸ'
punctuations = set([i for i in string.punctuation] + ['“‘”’'])
